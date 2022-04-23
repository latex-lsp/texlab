use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lsp_types::{CompletionParams, Position, TextDocumentIdentifier, TextDocumentPositionParams};
use texlab::{features::FeatureRequest, syntax::latex, DocumentLanguage, Uri, Workspace};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| latex::parse(black_box(include_str!("../texlab.tex"))));
    });

    c.bench_function("LaTeX/Completion/Command", |b| {
        let uri = Arc::new(Uri::parse("http://example.com/texlab.tex").unwrap());
        let text = Arc::new(include_str!("../texlab.tex").to_string());
        let mut workspace = Workspace::default();
        workspace
            .open(Arc::clone(&uri), text, DocumentLanguage::Latex)
            .unwrap();

        b.iter(|| {
            texlab::features::complete(FeatureRequest {
                params: CompletionParams {
                    context: None,
                    partial_result_params: Default::default(),
                    work_done_progress_params: Default::default(),
                    text_document_position: TextDocumentPositionParams::new(
                        TextDocumentIdentifier::new(uri.as_ref().clone().into()),
                        Position::new(0, 1),
                    ),
                },
                workspace: workspace.clone(),
                uri: Arc::clone(&uri),
            })
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
