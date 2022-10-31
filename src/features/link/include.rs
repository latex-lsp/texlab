use crate::{
    db::{document::Document, workspace::Workspace, Distro},
    Db,
};

use super::LinkBuilder;

pub(super) fn find_links(db: &dyn Db, document: Document, builder: &mut LinkBuilder) -> Option<()> {
    let workspace = Workspace::get(db);
    let distro = Distro::get(db);
    let parent = workspace
        .parents(db, Distro::get(db), document)
        .iter()
        .next()
        .copied()
        .unwrap_or(document);

    let graph = workspace.graph(db, parent, distro);
    for (target, origin) in graph
        .edges(db)
        .iter()
        .filter(|edge| edge.source(db) == document)
        .filter_map(|edge| edge.target(db).zip(edge.origin(db).into_explicit()))
    {
        builder.push(origin.link.range(db), target);
    }

    Some(())
}
