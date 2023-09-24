use base_db::{util::LineCol, Owner, Workspace};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use distro::Language;
use lsp_types::{ClientCapabilities, CompletionParams, Position, TextDocumentPositionParams, Url};
use parser::{parse_latex, SyntaxConfig};

const CODE: &str = include_str!("../../../texlab.tex");

fn criterion_benchmark(c: &mut Criterion) {
    let config = SyntaxConfig::default();
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| parse_latex(black_box(CODE), &config));
    });

    c.bench_function("LaTeX/Completion/Command_v2", |b| {
        let uri = Url::parse("http://example.com/texlab.tex").unwrap();
        let text = CODE.to_string();
        let mut workspace = Workspace::default();
        workspace.open(
            uri.clone(),
            text,
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        let client_capabilities = ClientCapabilities::default();
        let params = CompletionParams {
            context: None,
            text_document_position: TextDocumentPositionParams::new(
                lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                Position::new(0, 1),
            ),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        b.iter(|| {
            texlab::features::completion::complete(&workspace, &params, &client_capabilities, None)
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
