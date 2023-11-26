use std::path::Path;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Language {
    Tex,
    Bib,
    Aux,
    Log,
    Root,
    Latexmkrc,
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

        if name.eq_ignore_ascii_case(".latexmkrc") || name.eq_ignore_ascii_case("latexmkrc") {
            return Some(Self::Latexmkrc);
        }

        let extname = path.extension()?.to_str()?;
        match extname.to_lowercase().as_str() {
            "tex" | "sty" | "cls" | "def" | "lco" | "rnw" => Some(Self::Tex),
            "bib" | "bibtex" => Some(Self::Bib),
            "aux" => Some(Self::Aux),
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
