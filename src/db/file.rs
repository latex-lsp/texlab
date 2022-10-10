use std::path::PathBuf;

use lsp_types::Url;

use crate::Db;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Language {
    Tex,
    Bib,
    Log,
}

#[salsa::input]
pub struct FileId {
    #[return_ref]
    pub uri: Url,
}

#[salsa::tracked]
impl FileId {
    #[salsa::tracked(return_ref)]
    pub fn path(self, db: &dyn Db) -> Option<PathBuf> {
        let uri = self.uri(db);
        if uri.scheme() == "file" {
            uri.to_file_path().ok()
        } else {
            None
        }
    }

    #[salsa::tracked(return_ref)]
    pub fn stem(self, db: &dyn Db) -> Option<String> {
        let file_name = self.uri(db).path_segments()?.last()?;
        let file_stem = file_name
            .rfind('.')
            .map(|i| &file_name[..i])
            .unwrap_or(file_name);

        Some(file_stem.to_string())
    }

    #[salsa::tracked]
    pub fn language(self, db: &dyn Db) -> Option<Language> {
        let uri = self.uri(db);
        let (_, ext) = uri.path_segments()?.last()?.rsplit_once(".")?;
        match ext.to_lowercase().as_str() {
            "tex" | "sty" | "cls" | "def" | "lco" | "aux" | "rnw" => Some(Language::Tex),
            "bib" | "bibtex" => Some(Language::Bib),
            "log" => Some(Language::Log),
            _ => None,
        }
    }

    pub fn join(self, db: &dyn Db, path: &str) -> Result<FileId, url::ParseError> {
        let uri = self.uri(db);
        Ok(FileId::new(db, uri.join(path)?))
    }
}
