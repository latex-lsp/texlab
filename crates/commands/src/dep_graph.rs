use std::io::Write;

use anyhow::Result;
use base_db::Workspace;
use itertools::Itertools;
use rustc_hash::FxHashMap;

pub fn show_dependency_graph(workspace: &Workspace) -> Result<String> {
    let mut documents = FxHashMap::default();

    let mut writer = Vec::new();
    writeln!(&mut writer, "digraph G {{")?;
    writeln!(&mut writer, "rankdir = LR;")?;

    for (i, document) in workspace.iter().enumerate() {
        let node = format!("v{i:0>5}");

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
        documents.insert(&document.uri, node);
    }

    for edge in workspace
        .graphs()
        .values()
        .flat_map(|graph| &graph.edges)
        .unique()
    {
        let source = &documents[&edge.source];
        let target = &documents[&edge.target];
        let label = match &edge.data {
            base_db::deps::EdgeData::DirectLink(data) => &data.link.path.text,
            base_db::deps::EdgeData::AdditionalFiles => "<project>",
            base_db::deps::EdgeData::Artifact => "<artifact>",
            base_db::deps::EdgeData::FileList(_) => "<fls>",
        };

        writeln!(&mut writer, "\t{source} -> {target} [label=\"{label}\"];")?;
    }

    writeln!(&mut writer, "}}")?;
    Ok(String::from_utf8(writer)?)
}
