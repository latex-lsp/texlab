use futures::lock::Mutex;
use log::{debug, error, warn};
use petgraph::{graph::Graph, visit::Dfs};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error,
    ffi::OsStr,
    fmt,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use texlab_components::COMPONENT_DATABASE;
use texlab_protocol::{Options, TextDocumentItem, Uri};
use texlab_syntax::{bibtex, latex, LatexIncludeKind};
use texlab_tex::{DynamicDistribution, Language, Resolver};
use tokio::fs;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocumentParams<'a> {
    pub uri: Uri,
    pub text: String,
    pub language: Language,
    pub resolver: &'a Resolver,
    pub options: &'a Options,
    pub current_dir: &'a Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentContent {
    Latex(Box<latex::SymbolTable>),
    Bibtex(Box<bibtex::Tree>),
}

impl DocumentContent {
    pub fn as_latex(&self) -> Option<&latex::SymbolTable> {
        if let Self::Latex(table) = self {
            Some(&table)
        } else {
            None
        }
    }

    pub fn as_bibtex(&self) -> Option<&bibtex::Tree> {
        if let Self::Bibtex(tree) = self {
            Some(&tree)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub uri: Uri,
    pub text: String,
    pub content: DocumentContent,
    pub modified: SystemTime,
}

impl Document {
    pub fn is_file(&self) -> bool {
        self.uri.scheme() == "file"
    }

    pub fn open(params: DocumentParams) -> Self {
        let DocumentParams {
            uri,
            text,
            language,
            resolver,
            options,
            current_dir,
        } = params;

        let content = match language {
            Language::Latex => {
                let table = latex::open(latex::OpenParams {
                    uri: &uri,
                    text: &text,
                    resolver,
                    options,
                    current_dir,
                });
                DocumentContent::Latex(Box::new(table))
            }
            Language::Bibtex => {
                let tree = bibtex::open(&text);
                DocumentContent::Bibtex(Box::new(tree))
            }
        };

        Self {
            uri,
            text,
            content,
            modified: SystemTime::now(),
        }
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Document {}

impl Hash for Document {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct Snapshot(pub Vec<Arc<Document>>);

impl Snapshot {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, doc: Document) {
        self.0.push(Arc::new(doc));
    }

    pub fn find(&self, uri: &Uri) -> Option<Arc<Document>> {
        self.0.iter().find(|doc| doc.uri == *uri).map(Arc::clone)
    }

    pub fn relations(
        &self,
        uri: &Uri,
        options: &Options,
        current_dir: &Path,
    ) -> Vec<Arc<Document>> {
        let mut graph = Graph::new_undirected();
        let mut indices_by_uri = HashMap::new();
        for document in &self.0 {
            indices_by_uri.insert(&document.uri, graph.add_node(document));
        }

        for parent in &self.0 {
            if let DocumentContent::Latex(table) = &parent.content {
                table
                    .includes
                    .iter()
                    .flat_map(|include| include.all_targets.iter())
                    .filter_map(|targets| targets.iter().find_map(|target| self.find(target)))
                    .for_each(|child| {
                        graph.add_edge(indices_by_uri[&parent.uri], indices_by_uri[&child.uri], ());
                    });

                self.resolve_aux_targets(&parent.uri, options, current_dir, "aux")
                    .into_iter()
                    .flatten()
                    .find_map(|target| self.find(&target))
                    .into_iter()
                    .for_each(|child| {
                        graph.add_edge(indices_by_uri[&parent.uri], indices_by_uri[&child.uri], ());
                    });
            }
        }

        let mut documents = Vec::new();
        if self.find(uri).is_some() {
            let mut dfs = Dfs::new(&graph, indices_by_uri[uri]);
            while let Some(index) = dfs.next(&graph) {
                documents.push(Arc::clone(&graph[index]));
            }
        }
        documents
    }

    pub fn parent(
        &self,
        uri: &Uri,
        options: &Options,
        current_dir: &Path,
    ) -> Option<Arc<Document>> {
        for document in self.relations(uri, options, current_dir) {
            if let DocumentContent::Latex(table) = &document.content {
                if table.is_standalone {
                    return Some(document);
                }
            }
        }
        None
    }

    pub fn expand(&self, options: &Options, current_dir: &Path) -> Vec<Uri> {
        let mut unknown_targets = Vec::new();
        for parent in &self.0 {
            if let DocumentContent::Latex(table) = &parent.content {
                table
                    .includes
                    .iter()
                    .filter(|include| Self::should_expand_include(&table.tree, include))
                    .flat_map(|include| include.all_targets.iter())
                    .filter(|targets| targets.iter().all(|target| self.find(target).is_none()))
                    .flatten()
                    .for_each(|target| unknown_targets.push(target.clone()));

                self.resolve_aux_targets(&parent.uri, options, current_dir, "aux")
                    .into_iter()
                    .filter(|targets| targets.iter().all(|target| self.find(target).is_none()))
                    .flatten()
                    .for_each(|target| unknown_targets.push(target));
            }
        }
        unknown_targets
    }

    fn should_expand_include(tree: &latex::Tree, include: &latex::Include) -> bool {
        match include.kind {
            LatexIncludeKind::Bibliography | LatexIncludeKind::Latex => true,
            LatexIncludeKind::Everything
            | LatexIncludeKind::Image
            | LatexIncludeKind::Pdf
            | LatexIncludeKind::Svg => false,
            LatexIncludeKind::Package | LatexIncludeKind::Class => !include
                .paths(tree)
                .into_iter()
                .all(|name| COMPONENT_DATABASE.contains(name.text())),
        }
    }

    pub fn resolve_aux_targets(
        &self,
        tex_uri: &Uri,
        options: &Options,
        current_dir: &Path,
        extension: &str,
    ) -> Option<Vec<Uri>> {
        let mut targets = Vec::new();
        targets.push(tex_uri.with_extension(extension)?);
        if tex_uri.scheme() == "file" {
            let tex_path = tex_uri.to_file_path().ok()?;
            let file_stem = tex_path.file_stem()?;
            let aux_name = format!("{}.{}", file_stem.to_str()?, extension);

            if let Some(root_dir) = options
                .latex
                .as_ref()
                .and_then(|opts| opts.root_directory.as_ref())
            {
                let path = current_dir.join(root_dir).join(&aux_name);
                targets.push(Uri::from_file_path(path).ok()?);
            }

            if let Some(build_dir) = options
                .latex
                .as_ref()
                .and_then(|opts| opts.build.as_ref())
                .and_then(|opts| opts.output_directory.as_ref())
            {
                let path = current_dir.join(build_dir).join(&aux_name);
                targets.push(Uri::from_file_path(path).ok()?);
            }
        }
        Some(targets)
    }
}

#[derive(Debug)]
pub enum WorkspaceLoadError {
    UnknownLanguage,
    InvalidPath,
    IO(io::Error),
}

impl From<io::Error> for WorkspaceLoadError {
    fn from(why: io::Error) -> Self {
        Self::IO(why)
    }
}

impl fmt::Display for WorkspaceLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLanguage => write!(f, "Invalid language ID"),
            Self::InvalidPath => write!(f, "Invalid file path"),
            Self::IO(why) => write!(f, "{}", why),
        }
    }
}

impl error::Error for WorkspaceLoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::UnknownLanguage | Self::InvalidPath => None,
            Self::IO(why) => why.source(),
        }
    }
}

pub struct Workspace {
    distro: DynamicDistribution,
    current_dir: Arc<PathBuf>,
    snapshot: Mutex<Arc<Snapshot>>,
}

impl Workspace {
    pub fn new(distro: DynamicDistribution, current_dir: Arc<PathBuf>) -> Self {
        Self {
            distro,
            current_dir,
            snapshot: Mutex::default(),
        }
    }

    pub async fn get(&self) -> Arc<Snapshot> {
        let snapshot = self.snapshot.lock().await;
        Arc::clone(&snapshot)
    }

    pub async fn add(&self, document: TextDocumentItem, options: &Options) {
        let language = match Language::by_language_id(&document.language_id) {
            Some(language) => language,
            None => {
                error!(
                    "Invalid language id: {} ({})",
                    &document.language_id, &document.uri,
                );
                return;
            }
        };

        debug!("Adding document: {}", document.uri);
        let mut snapshot = self.snapshot.lock().await;
        *snapshot = self
            .add_or_update(
                &snapshot,
                document.uri.into(),
                document.text,
                language,
                options,
            )
            .await;
    }

    pub async fn load(&self, path: &Path, options: &Options) -> Result<(), WorkspaceLoadError> {
        let language = match path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(Language::by_extension)
        {
            Some(language) => language,
            None => {
                warn!("Could not determine language: {}", path.to_string_lossy());
                return Err(WorkspaceLoadError::UnknownLanguage);
            }
        };

        let uri = match Uri::from_file_path(path) {
            Ok(uri) => uri,
            Err(_) => {
                error!("Invalid path: {}", path.to_string_lossy());
                return Err(WorkspaceLoadError::InvalidPath);
            }
        };

        let text = match fs::read_to_string(path).await {
            Ok(text) => text,
            Err(why) => {
                warn!("Could not open file: {}", uri);
                return Err(WorkspaceLoadError::IO(why));
            }
        };

        debug!("Loading document: {}", uri);
        let mut snapshot = self.snapshot.lock().await;
        *snapshot = self
            .add_or_update(&snapshot, uri, text, language, options)
            .await;
        Ok(())
    }

    pub async fn update(&self, uri: Uri, text: String, options: &Options) {
        let mut snapshot = self.snapshot.lock().await;

        let old_document = match snapshot.0.iter().find(|x| x.uri == uri) {
            Some(document) => document,
            None => {
                warn!("Document not found: {}", uri);
                return;
            }
        };

        let language = match old_document.content {
            DocumentContent::Latex(_) => Language::Latex,
            DocumentContent::Bibtex(_) => Language::Bibtex,
        };

        *snapshot = self
            .add_or_update(&snapshot, uri, text, language, options)
            .await;
    }

    pub async fn reparse(&self, options: &Options) {
        let snapshot = self.get().await;
        for doc in &snapshot.0 {
            let language = match doc.content {
                DocumentContent::Latex(_) => Language::Latex,
                DocumentContent::Bibtex(_) => Language::Bibtex,
            };

            let mut snapshot = self.snapshot.lock().await;
            debug!("Reparsing document: {}", doc.uri);
            *snapshot = self
                .add_or_update(
                    &snapshot,
                    doc.uri.clone(),
                    doc.text.clone(),
                    language,
                    options,
                )
                .await;
        }
    }

    pub async fn detect_root(&self, uri: &Uri, options: &Options) -> io::Result<()> {
        if uri.scheme() != "file" {
            return Ok(());
        }

        if let Ok(mut path) = uri.to_file_path() {
            while path.pop() {
                let snapshot = self.get().await;
                if snapshot.parent(&uri, &options, &self.current_dir).is_some() {
                    break;
                }

                let mut entries = fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    if entry.file_type().await?.is_file() {
                        let path = entry.path();
                        if path
                            .extension()
                            .and_then(OsStr::to_str)
                            .and_then(Language::by_extension)
                            .is_some()
                        {
                            if let Ok(parent_uri) = Uri::from_file_path(&path) {
                                if snapshot.find(&parent_uri).is_none() {
                                    let _ = self.load(&path, options).await;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn detect_children(&self, options: &Options) {
        loop {
            let mut changed = false;

            let snapshot = self.get().await;
            for path in snapshot
                .expand(&options, &self.current_dir)
                .into_iter()
                .filter(|uri| uri.scheme() == "file")
                .filter_map(|uri| uri.to_file_path().ok())
            {
                if path.exists() {
                    changed |= self.load(&path, &options).await.is_ok();
                }
            }

            if !changed {
                break;
            }
        }
    }

    pub async fn reparse_all_if_newer(&self, options: &Options) {
        let snapshot = self.get().await;
        for doc in &snapshot.0 {
            if let Err(WorkspaceLoadError::IO(why)) = self.reparse_if_newer(doc, options).await {
                warn!("Reparsing document {} failed: {}", doc.uri, why);
            }
        }
    }

    async fn reparse_if_newer(
        &self,
        doc: &Document,
        options: &Options,
    ) -> Result<(), WorkspaceLoadError> {
        if !doc.is_file() {
            return Ok(());
        }

        if let Ok(path) = doc.uri.to_file_path() {
            let data = fs::metadata(&path).await?;
            if data.modified()? > doc.modified {
                self.load(&path, options).await?;
            }
        }
        Ok(())
    }

    async fn add_or_update(
        &self,
        snapshot: &Snapshot,
        uri: Uri,
        text: String,
        language: Language,
        options: &Options,
    ) -> Arc<Snapshot> {
        let resolver = self.distro.0.resolver().await;
        let document = Document::open(DocumentParams {
            uri,
            text,
            language,
            resolver: &resolver,
            options,
            current_dir: &self.current_dir,
        });

        let mut documents: Vec<Arc<Document>> = snapshot
            .0
            .iter()
            .filter(|x| x.uri != document.uri)
            .cloned()
            .collect();

        documents.push(Arc::new(document));
        Arc::new(Snapshot(documents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::env;
    use texlab_protocol::{LatexBuildOptions, LatexOptions};

    fn create_simple_document(uri: &Uri, language: Language, text: &str) -> Arc<Document> {
        Arc::new(Document::open(DocumentParams {
            uri: uri.clone(),
            text: text.into(),
            language,
            resolver: &Resolver::default(),
            options: &Options::default(),
            current_dir: &env::current_dir().unwrap(),
        }))
    }

    #[test]
    fn relations_append_missing_extension() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar/baz.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#"\include{bar/baz}"#),
            create_simple_document(&uri2, Language::Latex, r#""#),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri1, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri1, uri2]);
    }

    #[test]
    fn relations_parent_directory() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar/baz.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#""#),
            create_simple_document(&uri2, Language::Latex, r#"\input{../foo.tex}"#),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri1, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri1, uri2]);
    }

    #[test]
    fn relations_invalid_include() {
        let uri = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![create_simple_document(
            &uri,
            Language::Latex,
            r#"\include{<foo>?|bar|:}"#,
        )];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri]);
    }

    #[test]
    fn relations_bibliography() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar.bib").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#"\addbibresource{bar.bib}"#),
            create_simple_document(&uri2, Language::Bibtex, r#""#),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri2, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri2, uri1]);
    }

    #[test]
    fn relations_unknown_include() {
        let uri = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![create_simple_document(
            &uri,
            Language::Latex,
            r#"\input{bar.tex}"#,
        )];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri]);
    }

    #[test]
    fn relations_include_cycle() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#"\include{bar}"#),
            create_simple_document(&uri2, Language::Latex, r#"\input{foo.tex}"#),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri1, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri1, uri2]);
    }

    #[test]
    fn relations_same_parent() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar.tex").unwrap();
        let uri3 = Uri::parse("http://www.example.com/baz.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#"\input{bar.tex}\input{baz.tex}"#),
            create_simple_document(&uri2, Language::Latex, r#""#),
            create_simple_document(&uri3, Language::Latex, r#""#),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri3, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri3, uri1, uri2]);
    }

    #[test]
    fn relations_aux_default_options() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/foo.aux").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#""#),
            create_simple_document(&uri2, Language::Latex, r#""#),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri1, &Options::default(), &env::current_dir().unwrap())
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri1, uri2]);
    }

    #[test]
    fn relations_aux_output_directory() {
        let cwd = env::current_dir().unwrap();
        let options = Options {
            latex: Some(LatexOptions {
                build: Some(LatexBuildOptions {
                    output_directory: Some(PathBuf::from("build")),
                    ..LatexBuildOptions::default()
                }),
                ..LatexOptions::default()
            }),
            ..Options::default()
        };

        let uri1 = Uri::from_file_path(cwd.join("foo.tex")).unwrap();
        let uri2 = Uri::from_file_path(cwd.join("build/foo.aux")).unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            Arc::new(Document::open(DocumentParams {
                uri: uri1.clone(),
                text: String::new(),
                language: Language::Latex,
                resolver: &Resolver::default(),
                options: &options,
                current_dir: &cwd,
            })),
            Arc::new(Document::open(DocumentParams {
                uri: uri2.clone(),
                text: String::new(),
                language: Language::Latex,
                resolver: &Resolver::default(),
                options: &options,
                current_dir: &cwd,
            })),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri1, &options, &cwd)
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri1, uri2]);
    }

    #[test]
    fn relations_aux_root_directory() {
        let cwd = env::current_dir().unwrap();
        let options = Options {
            latex: Some(LatexOptions {
                root_directory: Some(PathBuf::from(".")),
                ..LatexOptions::default()
            }),
            ..Options::default()
        };

        let uri1 = Uri::from_file_path(cwd.join("src/foo.tex")).unwrap();
        let uri2 = Uri::from_file_path(cwd.join("foo.aux")).unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            Arc::new(Document::open(DocumentParams {
                uri: uri1.clone(),
                text: String::new(),
                language: Language::Latex,
                resolver: &Resolver::default(),
                options: &options,
                current_dir: &cwd,
            })),
            Arc::new(Document::open(DocumentParams {
                uri: uri2.clone(),
                text: String::new(),
                language: Language::Latex,
                resolver: &Resolver::default(),
                options: &options,
                current_dir: &cwd,
            })),
        ];
        let actual_uris: Vec<_> = snapshot
            .relations(&uri1, &options, &cwd)
            .into_iter()
            .map(|doc| doc.uri.clone())
            .collect();

        assert_eq!(actual_uris, vec![uri1, uri2]);
    }

    #[test]
    fn parent() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#""#),
            create_simple_document(
                &uri2,
                Language::Latex,
                r#"\begin{document}\include{foo}\end{document}"#,
            ),
        ];
        let doc = snapshot
            .parent(&uri1, &Options::default(), &env::current_dir().unwrap())
            .unwrap();
        assert_eq!(doc.uri, uri2);
    }

    #[test]
    fn parent_nothing_found() {
        let uri1 = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let uri2 = Uri::parse("http://www.example.com/bar.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![
            create_simple_document(&uri1, Language::Latex, r#""#),
            create_simple_document(&uri2, Language::Latex, r#"\begin{document}\end{document}"#),
        ];
        let doc = snapshot.parent(&uri1, &Options::default(), &env::current_dir().unwrap());
        assert_eq!(doc, None);
    }

    #[test]
    fn expand_aux_file() {
        let uri = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![create_simple_document(&uri, Language::Latex, r#""#)];
        let expansion = snapshot.expand(&Options::default(), &env::current_dir().unwrap());
        assert_eq!(
            expansion
                .iter()
                .map(|uri| uri.as_str())
                .filter(|uri| uri.ends_with(".aux"))
                .collect_vec(),
            vec!["http://www.example.com/foo.aux"]
        );
    }

    #[test]
    fn expand_local_package() {
        let uri = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![create_simple_document(
            &uri,
            Language::Latex,
            r#"\usepackage{foo-bar-baz}"#,
        )];
        let expansion = snapshot.expand(&Options::default(), &env::current_dir().unwrap());

        assert_eq!(
            expansion
                .iter()
                .map(|uri| uri.as_str())
                .filter(|uri| uri.ends_with(".sty"))
                .collect_vec(),
            vec!["http://www.example.com/foo-bar-baz.sty"]
        );
    }

    #[test]
    fn expand_system_package() {
        let uri = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![create_simple_document(
            &uri,
            Language::Latex,
            r#"\usepackage{amsmath}"#,
        )];
        let expansion = snapshot.expand(&Options::default(), &env::current_dir().unwrap());

        assert_eq!(
            expansion
                .iter()
                .map(|uri| uri.as_str())
                .filter(|uri| uri.ends_with(".sty"))
                .collect_vec(),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn expand_subdirectory() {
        let uri = Uri::parse("http://www.example.com/foo.tex").unwrap();
        let mut snapshot = Snapshot::new();
        snapshot.0 = vec![create_simple_document(
            &uri,
            Language::Latex,
            r#"\include{bar/baz}"#,
        )];
        let expansion = snapshot.expand(&Options::default(), &env::current_dir().unwrap());
        assert_eq!(
            expansion
                .iter()
                .map(|uri| uri.as_str())
                .filter(|uri| uri.ends_with(".tex"))
                .collect_vec(),
            vec!["http://www.example.com/bar/baz.tex"]
        );
    }
}
