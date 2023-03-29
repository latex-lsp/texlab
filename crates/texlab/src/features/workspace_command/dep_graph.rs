use anyhow::Result;
use itertools::Itertools;
use std::io::Write;

use rustc_hash::FxHashMap;

use crate::{
    db::{dependency_graph, Document, Workspace},
    Db,
};

pub fn show_dependency_graph(db: &dyn Db) -> Result<String> {
    let workspace = Workspace::get(db);

    let documents = workspace
        .documents(db)
        .iter()
        .enumerate()
        .map(|(i, doc)| (*doc, format!("v{i:0>5}")))
        .collect::<FxHashMap<Document, String>>();

    let mut writer = Vec::new();
    writeln!(&mut writer, "digraph G {{")?;
    writeln!(&mut writer, "rankdir = LR;")?;

    for (document, node) in &documents {
        let label = document.location(db).uri(db).as_str();
        let shape = if document.can_be_root(db) {
            "tripleoctagon"
        } else if document.can_be_built(db) {
            "doubleoctagon"
        } else {
            "octagon"
        };

        writeln!(&mut writer, "\t{node} [label=\"{label}\", shape={shape}];")?;
    }

    for edge in workspace
        .documents(db)
        .iter()
        .flat_map(|start| dependency_graph(db, *start).edges.iter())
        .unique()
    {
        let source = &documents[&edge.source];
        let target = &documents[&edge.target];
        let label = edge
            .origin
            .as_ref()
            .map_or("<artifact>", |origin| &origin.link.path(db).text(db));

        writeln!(&mut writer, "\t{source} -> {target} [label=\"{label}\"];")?;
    }

    writeln!(&mut writer, "}}")?;
    Ok(String::from_utf8(writer)?)
}
