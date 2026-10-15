use std::io::Write;

use anyhow::Result;
use base_db::{Workspace, deps::Edge};
use itertools::Itertools;
use rustc_hash::FxHashMap;

pub fn show_dependency_graph(workspace: &Workspace) -> Result<String> {
    let mut documents = FxHashMap::default();

    let mut writer = Vec::new();
    writeln!(&mut writer, "digraph G {{")?;
    writeln!(&mut writer, "rankdir = LR;")?;

    for (i, document) in workspace.iter().enumerate() {
        let node = format!("v{i:0>5}");

        let label = escape_dot_label(document.uri.as_str());
        let shape = if document
            .data
            .as_tex()
            .is_some_and(|data| data.semantics.can_be_root)
        {
            "tripleoctagon"
        } else if document
            .data
            .as_tex()
            .is_some_and(|data| data.semantics.can_be_compiled)
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
        .unique_by(|edge| (&edge.source, &edge.target, edge_label(&edge)))
    {
        let source = &documents[&edge.source];
        let target = &documents[&edge.target];
        let label = escape_dot_label(edge_label(edge));

        writeln!(&mut writer, "\t{source} -> {target} [label=\"{label}\"];")?;
    }

    writeln!(&mut writer, "}}")?;
    Ok(String::from_utf8(writer)?)
}

fn edge_label(edge: &Edge) -> &str {
    match &edge.data {
        base_db::deps::EdgeData::DirectLink(data) => &data.link.path.text,
        base_db::deps::EdgeData::AdditionalFiles => "<project>",
        base_db::deps::EdgeData::Artifact => "<artifact>",
        base_db::deps::EdgeData::FileList(_) => "<fls>",
    }
}

fn escape_dot_label(label: &str) -> String {
    let mut escaped = String::with_capacity(label.len());
    for character in label.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use test_utils::fixture::Fixture;

    use super::show_dependency_graph;
    use super::escape_dot_label;

    #[test]
    fn escapes_dot_string_special_characters() {
        assert_eq!(
            escape_dot_label("quote\" backslash\\ newline\n carriage\r"),
            r#"quote\" backslash\\ newline\n carriage\r"#,
        );
    }

    #[test]
    fn escapes_quotes_in_link_labels() {
        let fixture = Fixture::parse(
            r#"
%! main.tex
\input{child"q}
%! child"q.tex
"#,
        );
        let graph = show_dependency_graph(&fixture.workspace).unwrap();
        assert!(graph.contains("label=\"child\\\"q\""), "{graph}");
    }
}
