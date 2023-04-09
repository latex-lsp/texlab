use anyhow::Result;
use base_db::{graph, Document, Workspace};
use itertools::Itertools;
use std::io::Write;

use rustc_hash::FxHashMap;

pub fn show_dependency_graph(workspace: &Workspace) -> Result<String> {
    let documents = workspace
        .iter()
        .enumerate()
        .map(|(i, doc)| (doc, format!("v{i:0>5}")))
        .collect::<FxHashMap<&Document, String>>();

    let mut writer = Vec::new();
    writeln!(&mut writer, "digraph G {{")?;
    writeln!(&mut writer, "rankdir = LR;")?;

    for (document, node) in &documents {
        let label = document.uri.as_str();
        let shape = if document
            .data
            .as_tex()
            .map_or(false, |data| data.semantics.can_be_root)
        {
            "tripleoctagon"
        } else if document
            .data
            .as_tex()
            .map_or(false, |data| data.semantics.can_be_compiled)
        {
            "doubleoctagon"
        } else {
            "octagon"
        };

        writeln!(&mut writer, "\t{node} [label=\"{label}\", shape={shape}];")?;
    }

    for edge in workspace
        .iter()
        .flat_map(|start| graph::Graph::new(workspace, start).edges)
        .unique()
    {
        let source = &documents[edge.source];
        let target = &documents[edge.target];
        let label = edge
            .weight
            .as_ref()
            .map_or("<artifact>", |weight| &weight.link.path.text);

        writeln!(&mut writer, "\t{source} -> {target} [label=\"{label}\"];")?;
    }

    writeln!(&mut writer, "}}")?;
    Ok(String::from_utf8(writer)?)
}
