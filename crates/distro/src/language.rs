use std::path::Path;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Language {
    Tex,
    Bib,
    Log,
    Root,
    Tectonic,
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?;
        if name.eq_ignore_ascii_case(".texlabroot") || name.eq_ignore_ascii_case("texlabroot") {
            return Some(Self::Root);
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
            "texlabroot" => Some(Self::Root),
            _ => None,
        }
    }
}
