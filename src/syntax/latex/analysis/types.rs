use std::{borrow::Cow, sync::Arc};

use lsp_types::Url;
use rowan::TextRange;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

use crate::{distro::Resolver, Environment};

#[derive(Debug)]
pub struct LatexAnalyzerContext<'a> {
    pub environment: &'a Environment,
    pub document_uri: Arc<Url>,
    pub base_uri: Arc<Url>,
    pub extras: Extras,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Extras {
    pub implicit_links: ImplicitLinks,
    pub explicit_links: Vec<ExplicitLink>,
    pub command_names: FxHashSet<SmolStr>,
    pub environment_names: FxHashSet<String>,
    pub label_names: Vec<LabelName>,
    pub label_numbers_by_name: FxHashMap<String, String>,
    pub theorem_environments: Vec<TheoremEnvironment>,
    pub graphics_paths: FxHashSet<String>,
    pub can_be_built: bool,
    pub can_be_root: bool,
    pub has_document_environment: bool,
    pub has_subfiles_package: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct ImplicitLinks {
    pub aux: Vec<Arc<Url>>,
    pub log: Vec<Arc<Url>>,
    pub pdf: Vec<Arc<Url>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ExplicitLinkKind {
    Package,
    Class,
    Latex,
    Bibtex,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExplicitLink {
    pub stem: SmolStr,
    pub stem_range: TextRange,
    pub working_dir: Option<String>,
    pub targets: Vec<Arc<Url>>,
    pub kind: ExplicitLinkKind,
}

impl ExplicitLink {
    pub fn as_component_name(&self) -> Option<String> {
        match self.kind {
            ExplicitLinkKind::Package => Some(format!("{}.sty", self.stem)),
            ExplicitLinkKind::Class => Some(format!("{}.cls", self.stem)),
            ExplicitLinkKind::Latex | ExplicitLinkKind::Bibtex => None,
        }
    }

    pub fn targets<'a>(
        &'a self,
        base_dir: &'a Url,
        resolver: &'a Resolver,
    ) -> impl Iterator<Item = Url> + 'a {
        let dir = self
            .working_dir
            .as_ref()
            .and_then(|path| base_dir.join(path).ok())
            .map_or(Cow::Borrowed(base_dir), Cow::Owned);

        let suffixes: &[&str] = match self.kind {
            ExplicitLinkKind::Package => &[".sty"],
            ExplicitLinkKind::Class => &[".cls"],
            ExplicitLinkKind::Latex => &["", ".tex"],
            ExplicitLinkKind::Bibtex => &["", ".bib"],
        };

        suffixes
            .iter()
            .filter_map(move |suffix| {
                let file_name = format!("{}{}", self.stem, suffix);
                dir.join(&file_name).ok()
            })
            .chain(suffixes.iter().filter_map(move |suffix| {
                let file_name = format!("{}{}", self.stem, suffix);
                resolver
                    .files_by_name
                    .get(file_name.as_str())
                    .and_then(|path| Url::from_file_path(path).ok())
            }))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct TheoremEnvironment {
    pub name: String,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct LabelName {
    pub text: SmolStr,
    pub range: TextRange,
    pub is_definition: bool,
}
