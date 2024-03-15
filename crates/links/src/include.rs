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

    let graph = &params.workspace.graphs()[&parent.uri];

    for edge in &graph.edges {
        if edge.source == document.uri {
            if let EdgeData::DirectLink(data) = &edge.data {
                let target = params.workspace.lookup(&edge.target).unwrap();
                results.push(DocumentLocation::new(target, data.link.path.range));
            }
        }
    }

    Some(())
}
