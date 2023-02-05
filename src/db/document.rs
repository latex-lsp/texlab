use std::path::{Path, PathBuf};

use lsp_types::Url;
use rowan::{TextRange, TextSize};

use crate::{
    db::{
        diagnostics::Diagnostic,
        parse::{BibDocumentData, LogDocumentData, TectonicData, TexDocumentData, TexlabRootData},
    },
    parser::{parse_bibtex, parse_build_log, parse_latex},
    util::line_index::LineIndex,
    Db,
};

use super::{
    analysis::TexLinkKind,
    parse::{self, DocumentData},
};

#[salsa::interned]
pub struct Location {
    #[return_ref]
    pub uri: Url,
}

#[salsa::tracked]
impl Location {
    #[salsa::tracked(return_ref)]
    pub fn path(self, db: &dyn Db) -> Option<PathBuf> {
        let uri = self.uri(db);
        if uri.scheme() == "file" {
            uri.to_file_path().ok()
        } else {
            None
        }
    }

    pub fn stem(self, db: &dyn Db) -> Option<String> {
        let path = self.uri(db).to_file_path().ok()?;
        let stem = path.file_stem()?.to_str()?;
        Some(String::from(stem))
    }

    pub fn join(self, db: &dyn Db, path: &str) -> Option<Location> {
        let uri = self.uri(db).join(path).ok()?;
        Some(Location::new(db, uri))
    }
}

#[salsa::input]
pub struct Contents {
    #[return_ref]
    pub text: String,
}

#[salsa::tracked]
impl Contents {
    #[salsa::tracked(return_ref)]
    pub fn line_index(self, db: &dyn Db) -> LineIndex {
        LineIndex::new(self.text(db))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Owner {
    Client,
    Server,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Language {
    Tex,
    Bib,
    Log,
    TexlabRoot,
    Tectonic,
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?;
        if name.eq_ignore_ascii_case(".texlabroot") || name.eq_ignore_ascii_case("texlabroot") {
            return Some(Self::TexlabRoot);
        }

        if name.eq_ignore_ascii_case("Tectonic.toml") {
            return Some(Self::Tectonic);
        }

        let extname = path.extension()?.to_str()?;
        match extname.to_lowercase().as_str() {
            "tex" | "sty" | "cls" | "def" | "lco" | "aux" | "rnw" => Some(Self::Tex),
            "bib" | "bibtex" => Some(Self::Bib),
            "log" => Some(Self::Log),
            _ => None,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "tex" | "latex" => Some(Self::Tex),
            "bib" | "bibtex" => Some(Self::Bib),
            "texlabroot" => Some(Self::TexlabRoot),
            _ => None,
        }
    }
}

#[salsa::input]
pub struct LinterData {
    #[return_ref]
    pub chktex: Vec<Diagnostic>,
}

#[salsa::input]
pub struct Document {
    pub location: Location,
    pub contents: Contents,
    pub language: Language,
    pub owner: Owner,
    pub cursor: TextSize,
    pub linter: LinterData,
}

impl Document {
    pub fn edit(self, db: &mut dyn Db, range: TextRange, replace_with: &str) {
        let mut text = self.contents(db).set_text(db).to(String::new());
        text.replace_range(std::ops::Range::<usize>::from(range), &replace_with);
        self.contents(db).set_text(db).to(text);
        self.set_cursor(db).to(range.start());
    }

    pub fn directory(self, db: &dyn Db) -> Location {
        self.location(db).join(db, ".").unwrap()
    }

    pub fn ancestor_dirs<'db>(self, db: &'db dyn Db) -> impl Iterator<Item = &'db Path> + 'db {
        self.location(db)
            .path(db)
            .as_deref()
            .into_iter()
            .flat_map(|path| path.ancestors())
            .skip(1)
    }
}

#[salsa::tracked]
impl Document {
    #[salsa::tracked]
    pub fn parse(self, db: &dyn Db) -> DocumentData {
        let text = self.contents(db).text(db);
        match self.language(db) {
            Language::Tex => {
                let data = TexDocumentData::new(db, parse_latex(text));
                parse::DocumentData::Tex(data)
            }
            Language::Bib => {
                let data = BibDocumentData::new(db, parse_bibtex(text));
                DocumentData::Bib(data)
            }
            Language::Log => {
                let data = LogDocumentData::new(db, parse_build_log(text));
                DocumentData::Log(data)
            }
            Language::TexlabRoot => {
                let data = TexlabRootData;
                DocumentData::TexlabRoot(data)
            }
            Language::Tectonic => {
                let data = TectonicData;
                DocumentData::Tectonic(data)
            }
        }
    }

    #[salsa::tracked]
    pub fn can_be_root(self, db: &dyn Db) -> bool {
        self.parse(db).as_tex().map_or(false, |data| {
            let analysis = data.analyze(db);
            analysis.has_document_environment(db)
                || !analysis
                    .links(db)
                    .iter()
                    .filter(|link| link.kind(db) == TexLinkKind::Cls)
                    .any(|link| link.path(db).text(db) == "subfiles")
        })
    }

    #[salsa::tracked]
    pub fn can_be_built(self, db: &dyn Db) -> bool {
        self.parse(db)
            .as_tex()
            .map_or(false, |data| data.analyze(db).has_document_environment(db))
    }
}
