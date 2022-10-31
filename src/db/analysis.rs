use lsp_types::Url;
use rowan::{ast::AstNode, TextRange};

use crate::{syntax::latex, Db};

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
pub struct TexLabelName {
    pub kind: TexLabelKind,
    pub name: Word,
    pub range: TextRange,
}

impl TexLabelName {
    fn of_definition(db: &dyn Db, node: latex::SyntaxNode, results: &mut Vec<Self>) -> Option<()> {
        let label = latex::LabelDefinition::cast(node)?;
        let name = label.name()?.key()?;
        results.push(TexLabelName::new(
            db,
            TexLabelKind::Definition,
            Word::new(db, name.to_string()),
            latex::small_range(&name),
        ));

        Some(())
    }

    fn of_reference(db: &dyn Db, node: latex::SyntaxNode, results: &mut Vec<Self>) -> Option<()> {
        let label = latex::LabelReference::cast(node)?;
        for name in label.name_list()?.keys() {
            results.push(TexLabelName::new(
                db,
                TexLabelKind::Reference,
                Word::new(db, name.to_string()),
                latex::small_range(&name),
            ));
        }

        Some(())
    }

    fn of_reference_range(
        db: &dyn Db,
        node: latex::SyntaxNode,
        results: &mut Vec<Self>,
    ) -> Option<()> {
        let label = latex::LabelReferenceRange::cast(node)?;
        if let Some(name) = label.from().and_then(|name| name.key()) {
            results.push(TexLabelName::new(
                db,
                TexLabelKind::Reference,
                Word::new(db, name.to_string()),
                latex::small_range(&name),
            ));
        }

        if let Some(name) = label.to().and_then(|name| name.key()) {
            results.push(TexLabelName::new(
                db,
                TexLabelKind::Reference,
                Word::new(db, name.to_string()),
                latex::small_range(&name),
            ));
        }

        Some(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum TexLabelKind {
    Definition,
    Reference,
}

#[salsa::tracked]
pub struct TexAnalysis {
    #[return_ref]
    pub links: Vec<TexLink>,

    #[return_ref]
    pub labels: Vec<TexLabelName>,
}

impl TexAnalysis {
    pub(super) fn analyze(db: &dyn Db, root: &latex::SyntaxNode) -> Self {
        let mut links = Vec::new();
        let mut labels = Vec::new();

        for node in root.descendants() {
            TexLink::of_include(db, node.clone(), &mut links)
                .or_else(|| TexLink::of_import(db, node.clone(), &mut links))
                .or_else(|| TexLabelName::of_definition(db, node.clone(), &mut labels))
                .or_else(|| TexLabelName::of_reference(db, node.clone(), &mut labels))
                .or_else(|| TexLabelName::of_reference_range(db, node.clone(), &mut labels));
        }

        Self::new(db, links, labels)
    }
}
