use std::sync::Arc;

use cstree::TextRange;
use derive_more::From;
use smol_str::SmolStr;
use variantly::Variantly;

use crate::{
    syntax::{bibtex, latex, CstNode},
    DocumentLanguage,
};

use super::{Document, DocumentDatabase};

#[derive(Debug, Clone)]
pub enum Tree {
    Latex(latex::SyntaxNode<LatexAnnotation>),
    Bibtex(bibtex::SyntaxNode<BibtexAnnotation>),
    BuildLog,
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Latex(tree1), Self::Latex(tree2)) => (tree1.green() == tree2.green()),
            (Self::Bibtex(tree1), Self::Bibtex(tree2)) => (tree1.green() == tree2.green()),
            (Self::BuildLog, Self::BuildLog) => true,
            _ => false,
        }
    }
}

impl Eq for Tree {}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, From, Variantly)]
pub enum BibtexAnnotation {}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, From, Variantly)]
pub enum LatexAnnotation {
    Include(LatexInclude),
    Import(LatexImport),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum LatexIncludeKind {
    Package,
    Class,
    Latex,
    Bibtex,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexIncludePath(salsa::InternId);

impl salsa::InternKey for LatexIncludePath {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexIncludePathData {
    pub kind: LatexIncludeKind,
    pub range: TextRange,
    pub text: SmolStr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexInclude(salsa::InternId);

impl salsa::InternKey for LatexInclude {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexIncludeData {
    pub kind: LatexIncludeKind,
    pub paths: Vec<LatexIncludePath>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexImport(salsa::InternId);

impl salsa::InternKey for LatexImport {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexImportData {
    pub directory: (TextRange, SmolStr),
    pub file: (TextRange, SmolStr),
}

#[salsa::query_group(ParserDatabaseStorage)]
pub trait ParserDatabase: DocumentDatabase {
    fn tree(&self, document: Document) -> Tree;

    #[salsa::interned]
    fn intern_latex_include(&self, data: LatexIncludeData) -> LatexInclude;

    #[salsa::interned]
    fn intern_latex_include_path(&self, data: LatexIncludePathData) -> LatexIncludePath;

    #[salsa::interned]
    fn intern_latex_import(&self, data: LatexImportData) -> LatexImport;

    fn latex_annotations(&self, document: Document) -> Arc<Vec<LatexAnnotation>>;

    fn latex_include_paths(&self, document: Document) -> Arc<Vec<LatexIncludePath>>;

    fn latex_imports(&self, document: Document) -> Arc<Vec<LatexImport>>;
}

fn tree(db: &dyn ParserDatabase, document: Document) -> Tree {
    let text = db.text(document);
    match db.language(document) {
        DocumentLanguage::Latex => {
            let root = latex::parse(text.as_str()).root;
            for node in root.descendants() {
                analyze_latex_include(db, node).or_else(|| analyze_latex_import(db, node));
            }
            Tree::Latex(root)
        }
        DocumentLanguage::Bibtex => {
            let root = bibtex::parse(text.as_str()).root;
            Tree::Bibtex(root)
        }
        DocumentLanguage::BuildLog => Tree::BuildLog,
    }
}

fn analyze_latex_include(
    db: &dyn ParserDatabase,
    node: &latex::SyntaxNode<LatexAnnotation>,
) -> Option<()> {
    let include = latex::Include::cast(node)?;
    let kind = match include.syntax().kind() {
        latex::PACKAGE_INCLUDE => LatexIncludeKind::Package,
        latex::CLASS_INCLUDE => LatexIncludeKind::Class,
        latex::LATEX_INCLUDE => LatexIncludeKind::Latex,
        latex::BIBTEX_INCLUDE | latex::BIBLATEX_INCLUDE => LatexIncludeKind::Bibtex,
        _ => return Some(()),
    };

    let paths = include
        .path_list()
        .into_iter()
        .flat_map(|list| list.words())
        .map(|path| {
            db.intern_latex_include_path(LatexIncludePathData {
                kind,
                range: path.text_range(),
                text: path.text().into(),
            })
        })
        .collect();

    node.set_data(
        db.intern_latex_include(LatexIncludeData { kind, paths })
            .into(),
    );
    Some(())
}

fn analyze_latex_import(
    db: &dyn ParserDatabase,
    node: &latex::SyntaxNode<LatexAnnotation>,
) -> Option<()> {
    let import = latex::Import::cast(node)?;
    let directory = import.directory()?.word()?;
    let file = import.file()?.word()?;
    node.set_data(
        db.intern_latex_import(LatexImportData {
            directory: (directory.text_range(), directory.text().into()),
            file: (file.text_range(), file.text().into()),
        })
        .into(),
    );
    Some(())
}

fn latex_annotations(db: &dyn ParserDatabase, document: Document) -> Arc<Vec<LatexAnnotation>> {
    match db.tree(document) {
        Tree::Latex(tree) => Arc::new(
            tree.descendants()
                .filter_map(|node| node.get_data())
                .map(|ann| *ann.as_ref())
                .collect(),
        ),
        Tree::Bibtex(_) | Tree::BuildLog => Arc::default(),
    }
}

fn latex_include_paths(db: &dyn ParserDatabase, document: Document) -> Arc<Vec<LatexIncludePath>> {
    Arc::new(
        db.latex_annotations(document)
            .iter()
            .filter_map(|ann| ann.include())
            .flat_map(|include| db.lookup_intern_latex_include(include).paths)
            .collect(),
    )
}

fn latex_imports(db: &dyn ParserDatabase, document: Document) -> Arc<Vec<LatexImport>> {
    Arc::new(
        db.latex_annotations(document)
            .iter()
            .filter_map(|ann| ann.import())
            .collect(),
    )
}
