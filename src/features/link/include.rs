use crate::{
    db::{dependency_graph, document::Document, workspace::Workspace},
    Db,
};

use super::LinkBuilder;

pub(super) fn find_links(db: &dyn Db, document: Document, builder: &mut LinkBuilder) -> Option<()> {
    let workspace = Workspace::get(db);
    let parent = workspace
        .parents(db, document)
        .iter()
        .next()
        .copied()
        .unwrap_or(document);

    let graph = dependency_graph(db, parent);
    for edge in graph.edges.iter().filter(|edge| edge.source == document) {
        if let Some(origin) = edge.origin {
            builder.push(origin.link.range(db), edge.target);
        }
    }

    Some(())
}
