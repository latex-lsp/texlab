use std::sync::Arc;

use brunch::{benches, Bench};
use lsp_types::{
    CompletionParams, Position, TextDocumentIdentifier, TextDocumentPositionParams, Url,
};
use texlab::{features::FeatureRequest, syntax::latex, DocumentLanguage, Workspace};

benches!(
    Bench::new("parser::latex", "texlab.tex")
        .with(|| { latex::parse(include_str!("../texlab.tex")) }),
    Bench::new("completion::latex", "command").with(|| {
        let uri = Arc::new(Url::parse("http://example.com/texlab.tex").unwrap());
        let text = Arc::new(include_str!("../texlab.tex").to_string());
        let mut workspace = Workspace::default();
        workspace
            .open(Arc::clone(&uri), text, DocumentLanguage::Latex)
            .unwrap();

        texlab::features::complete(FeatureRequest {
            params: CompletionParams {
                context: None,
                partial_result_params: Default::default(),
                work_done_progress_params: Default::default(),
                text_document_position: TextDocumentPositionParams::new(
                    TextDocumentIdentifier::new(uri.as_ref().clone()),
                    Position::new(0, 1),
                ),
            },
            workspace: workspace.clone(),
            uri: Arc::clone(&uri),
        })
    }),
);
