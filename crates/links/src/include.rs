use base_db::{DocumentLocation, FeatureParams};

pub(super) fn find_links<'a>(
    params: &FeatureParams<'a>,
    results: &mut Vec<DocumentLocation<'a>>,
) -> Option<()> {
    let document = params.document;
    let parent = *params
        .workspace
        .parents(document)
        .iter()
        .next()
        .unwrap_or(&document);

    let graph = base_db::graph::Graph::new(params.workspace, parent);

    for edge in &graph.edges {
        if edge.source == document {
            if let Some(weight) = &edge.weight {
                results.push(DocumentLocation::new(edge.target, weight.link.path.range));
            }
        }
    }

    Some(())
}
