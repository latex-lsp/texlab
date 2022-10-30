use itertools::Itertools;

use crate::Db;

use super::{
    analysis::TexLink,
    document::{Document, Location},
    Distro,
};

#[salsa::tracked]
pub struct Resolved {
    pub source: Document,
    pub target: Option<Document>,
    pub origin: Origin,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Origin {
    Explicit(Explicit),
    Implicit(Implicit),
}

impl Origin {
    pub fn into_explicit(self) -> Option<Explicit> {
        match self {
            Self::Explicit(data) => Some(data),
            Self::Implicit(_) => None,
        }
    }

    pub fn into_implicit(self) -> Option<Implicit> {
        match self {
            Self::Explicit(_) => None,
            Self::Implicit(data) => Some(data),
        }
    }

    pub fn into_locations(self, db: &dyn Db, distro: Distro) -> &Vec<Location> {
        match self {
            Self::Explicit(data) => data.link.locations(db, data.base_dir, distro),
            Self::Implicit(data) => data.locations(db),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Explicit {
    pub link: TexLink,
    pub base_dir: Location,
}

#[salsa::tracked]
pub struct Implicit {
    #[return_ref]
    pub locations: Vec<Location>,
}

#[salsa::tracked]
pub struct Graph {
    #[return_ref]
    pub items: Vec<Resolved>,
    pub start: Document,
}

#[salsa::tracked]
impl Graph {
    #[salsa::tracked(return_ref)]
    pub fn preorder(self, db: &dyn Db) -> Vec<Document> {
        std::iter::once(self.start(db))
            .chain(self.items(db).iter().filter_map(|group| group.target(db)))
            .unique()
            .collect()
    }
}
