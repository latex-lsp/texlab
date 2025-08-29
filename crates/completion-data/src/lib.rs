use std::{borrow::Cow, io::Read};

use flate2::read::GzDecoder;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database<'a> {
    #[serde(rename = "components", borrow)]
    packages: Vec<Package<'a>>,

    #[serde(borrow)]
    metadata: Vec<Metadata<'a>>,

    #[serde(skip)]
    lookup_packages: FxHashMap<&'a str, usize>,

    #[serde(skip)]
    lookup_metadata: FxHashMap<&'a str, usize>,

    #[serde(skip)]
    lookup_kernel: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package<'a> {
    #[serde(borrow)]
    pub file_names: Vec<&'a str>,

    #[serde(borrow)]
    pub references: Vec<&'a str>,

    #[serde(borrow)]
    pub commands: Vec<Command<'a>>,

    #[serde(borrow)]
    pub environments: Vec<&'a str>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command<'a> {
    pub name: Cow<'a, str>,

    #[serde(borrow)]
    pub image: Option<&'a str>,

    #[serde(borrow)]
    pub glyph: Option<Cow<'a, str>>,

    #[serde(borrow)]
    pub parameters: Vec<Parameter<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter<'a>(#[serde(borrow)] pub Vec<Argument<'a>>);

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Argument<'a> {
    pub name: &'a str,

    #[serde(borrow)]
    pub image: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata<'a> {
    pub name: &'a str,

    #[serde(borrow)]
    pub caption: Option<Cow<'a, str>>,

    #[serde(borrow)]
    pub description: Option<Cow<'a, str>>,
}

impl<'a> Database<'a> {
    pub fn iter(&'_ self) -> impl Iterator<Item = &'_ Package<'_>> + '_ {
        self.packages.iter()
    }

    pub fn find(&'_ self, name: &str) -> Option<&'_ Package<'_>> {
        self.lookup_packages
            .get(name)
            .map(|index| &self.packages[*index])
    }

    pub fn meta(&'_ self, name: &str) -> Option<&'_ Metadata<'_>> {
        self.lookup_metadata
            .get(name)
            .map(|index| &self.metadata[*index])
    }

    pub fn kernel(&'_ self) -> &'a Package<'_> {
        &self.packages[self.lookup_kernel]
    }
}

const JSON_GZ: &[u8] = include_bytes!("../data/completion.json.gz");

pub static DATABASE: Lazy<Database<'static>> = Lazy::new(|| {
    let mut decoder = GzDecoder::new(JSON_GZ);
    let json = Box::leak(Box::default());
    decoder.read_to_string(json).unwrap();
    let mut db: Database = serde_json::from_str(json).unwrap();
    db.lookup_packages = db
        .packages
        .iter()
        .enumerate()
        .flat_map(|(i, pkg)| pkg.file_names.iter().map(move |name| (*name, i)))
        .collect();

    db.lookup_metadata = db
        .metadata
        .iter()
        .enumerate()
        .unique_by(|(_, meta)| meta.name)
        .map(|(i, meta)| (meta.name, i))
        .collect();

    db.lookup_kernel = db
        .packages
        .iter()
        .position(|package| package.file_names.is_empty())
        .unwrap();

    db
});

pub fn included_packages<'a>(
    params: &'a base_db::FeatureParams<'a>,
) -> impl Iterator<Item = &'static crate::Package<'static>> + 'a {
    let db = &crate::DATABASE;
    let documents = params.project.documents.iter();
    let links = documents
        .filter_map(|document| document.data.as_tex())
        .flat_map(|data| data.semantics.links.iter());

    links
        .filter_map(|link| link.package_name())
        .filter_map(|name| db.find(&name))
        .chain(std::iter::once(db.kernel()))
        .flat_map(|pkg| {
            pkg.references
                .iter()
                .filter_map(|name| db.find(name))
                .chain(std::iter::once(pkg))
        })
}
