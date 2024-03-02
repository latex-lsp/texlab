use base_db::{
    deps::{self, EdgeData},
    DocumentLocation, FeatureParams,
};

pub(super) fn find_links<'a>(
    params: &FeatureParams<'a>,
    results: &mut Vec<DocumentLocation<'a>>,
) -> Option<()> {
    let document = params.document;
    let parent = *deps::parents(params.workspace, document)
        .iter()
        .next()
        .unwrap_or(&document);

    let graph = deps::Graph::new(params.workspace, parent);

    for edge in &graph.edges {
        if edge.source == document {
            if let EdgeData::DirectLink(data) = &edge.data {
                results.push(DocumentLocation::new(edge.target, data.link.path.range));
            }
        }
    }

    Some(())
}
