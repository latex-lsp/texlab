use super::LinkBuilder;

pub(super) fn find_links(builder: &mut LinkBuilder) -> Option<()> {
    let parent = *builder
        .workspace
        .parents(builder.document)
        .iter()
        .next()
        .unwrap_or(&builder.document);

    let graph = base_db::graph::Graph::new(builder.workspace, parent);

    for edge in &graph.edges {
        if edge.source == builder.document {
            if let Some(weight) = &edge.weight {
                builder.push(weight.link.path.range, edge.target);
            }
        }
    }

    Some(())
}
