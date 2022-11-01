pub mod label;

use lsp_types::Url;
use rowan::{ast::AstNode, TextRange};

use crate::{
    syntax::latex::{self, HasCurly},
    Db,
};

use super::{document::Location, Distro, Word};

#[salsa::tracked]
pub struct TexLink {
    pub kind: TexLinkKind,
    pub path: Word,
    pub range: TextRange,
    pub working_dir: Option<Word>,
}

#[salsa::tracked]
impl TexLink {
    #[salsa::tracked(return_ref)]
    pub fn locations(self, db: &dyn Db, base_dir: Location, distro: Distro) -> Vec<Location> {
        let stem = self.path(db).text(db);
        let paths = self
            .kind(db)
            .extensions()
            .iter()
            .map(|ext| format!("{stem}.{ext}"));

        let file_name_db = distro.file_name_db(db);
        let distro_files = std::iter::once(stem.to_string())
            .chain(paths.clone())
            .filter_map(|path| file_name_db.get(path.as_str()))
            .flat_map(|path| Url::from_file_path(path))
            .map(|uri| Location::new(db, uri));

        std::iter::once(stem.to_string())
            .chain(paths)
            .flat_map(|path| base_dir.uri(db).join(&path))
            .map(|uri| Location::new(db, uri))
            .chain(distro_files)
            .collect()
    }
}

impl TexLink {
    fn of_include(db: &dyn Db, node: latex::SyntaxNode, results: &mut Vec<Self>) -> Option<()> {
        let include = latex::Include::cast(node)?;
        let kind = match include.syntax().kind() {
            latex::LATEX_INCLUDE => TexLinkKind::Tex,
            latex::BIBLATEX_INCLUDE | latex::BIBTEX_INCLUDE => TexLinkKind::Bib,
            latex::PACKAGE_INCLUDE => TexLinkKind::Sty,
            latex::CLASS_INCLUDE => TexLinkKind::Cls,
            _ => return None,
        };

        for path in include.path_list()?.keys() {
            results.push(Self::new(
                db,
                kind,
                Word::new(db, path.to_string()),
                latex::small_range(&path),
                None,
            ));
        }

        Some(())
    }

    fn of_import(db: &dyn Db, node: latex::SyntaxNode, results: &mut Vec<Self>) -> Option<()> {
        let import = latex::Import::cast(node)?;
        let working_dir = import.directory()?.key()?;
        let path = import.file()?.key()?;
        results.push(Self::new(
            db,
            TexLinkKind::Tex,
            Word::new(db, path.to_string()),
            latex::small_range(&path),
            Some(Word::new(db, working_dir.to_string())),
        ));

        Some(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum TexLinkKind {
    Sty,
    Cls,
    Tex,
    Bib,
}

impl TexLinkKind {
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Sty => &["sty"],
            Self::Cls => &["cls"],
            Self::Tex => &["tex"],
            Self::Bib => &["bib"],
        }
    }
}

#[salsa::tracked]
pub struct TheoremEnvironment {
    pub name: Word,
    pub description: Word,
}

impl TheoremEnvironment {
    fn of_definition(db: &dyn Db, node: latex::SyntaxNode, results: &mut Vec<Self>) -> Option<()> {
        let theorem = latex::TheoremDefinition::cast(node)?;
        let name = theorem.name()?.key()?.to_string();
        let description = theorem.description()?;
        let description = description.content_text()?;

        results.push(Self::new(
            db,
            Word::new(db, name),
            Word::new(db, description),
        ));

        Some(())
    }
}

#[salsa::tracked]
pub struct GraphicsPath {
    #[return_ref]
    pub path: String,
}

impl GraphicsPath {
    pub fn of_command(db: &dyn Db, node: latex::SyntaxNode, results: &mut Vec<Self>) -> Option<()> {
        let definition = latex::GraphicsPath::cast(node)?;
        for path in definition
            .path_list()
            .filter_map(|group| group.key())
            .map(|path| path.to_string())
        {
            results.push(GraphicsPath::new(db, path));
        }

        Some(())
    }
}

#[salsa::tracked]
pub struct TexAnalysis {
    #[return_ref]
    pub links: Vec<TexLink>,

    #[return_ref]
    pub labels: Vec<label::Name>,

    #[return_ref]
    pub label_numbers: Vec<label::Number>,

    #[return_ref]
    pub theorem_environments: Vec<TheoremEnvironment>,

    #[return_ref]
    pub graphics_paths: Vec<GraphicsPath>,

    #[return_ref]
    pub command_name_ranges: Vec<TextRange>,

    #[return_ref]
    pub environment_names: Vec<String>,
}

impl TexAnalysis {
    pub(super) fn analyze(db: &dyn Db, root: &latex::SyntaxNode) -> Self {
        let mut links = Vec::new();
        let mut labels = Vec::new();
        let mut label_numbers = Vec::new();
        let mut theorem_environments = Vec::new();
        let mut graphics_paths = Vec::new();
        let mut command_name_ranges = Vec::new();
        let mut environment_names = Vec::new();

        for node in root.descendants() {
            TexLink::of_include(db, node.clone(), &mut links)
                .or_else(|| TexLink::of_import(db, node.clone(), &mut links))
                .or_else(|| label::Name::of_definition(db, node.clone(), &mut labels))
                .or_else(|| label::Name::of_reference(db, node.clone(), &mut labels))
                .or_else(|| label::Name::of_reference_range(db, node.clone(), &mut labels))
                .or_else(|| label::Number::of_number(db, node.clone(), &mut label_numbers))
                .or_else(|| {
                    TheoremEnvironment::of_definition(db, node.clone(), &mut theorem_environments)
                })
                .or_else(|| GraphicsPath::of_command(db, node.clone(), &mut graphics_paths))
                .or_else(|| {
                    let range = latex::GenericCommand::cast(node.clone())?
                        .name()?
                        .text_range();

                    command_name_ranges.push(range);
                    Some(())
                })
                .or_else(|| {
                    let begin = latex::Begin::cast(node.clone())?;
                    environment_names.push(begin.name()?.key()?.to_string());
                    Some(())
                });
        }

        Self::new(
            db,
            links,
            labels,
            label_numbers,
            theorem_environments,
            graphics_paths,
            command_name_ranges,
            environment_names,
        )
    }

    pub fn find_label_number(self, db: &dyn Db, name: Word) -> Option<Word> {
        self.label_numbers(db)
            .iter()
            .find(|number| number.name(db) == name)
            .map(|number| number.text(db))
    }
}
