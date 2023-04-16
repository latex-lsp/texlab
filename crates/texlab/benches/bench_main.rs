use base_db::{Owner, Workspace};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use distro::Language;
use lsp_types::{ClientCapabilities, Position, Url};
use parser::{parse_latex, SyntaxConfig};
use rowan::TextSize;

const CODE: &str = include_str!("../../../texlab.tex");

fn criterion_benchmark(c: &mut Criterion) {
    let config = SyntaxConfig::default();
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| parse_latex(black_box(CODE), &config));
    });

    c.bench_function("LaTeX/Completion/Command", |b| {
        let uri = Url::parse("http://example.com/texlab.tex").unwrap();
        let text = CODE.to_string();
        let mut workspace = Workspace::default();
        workspace.open(
            uri.clone(),
            text,
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        let client_capabilities = ClientCapabilities::default();

        b.iter(|| {
            texlab::features::completion::complete(
                &workspace,
                &uri,
                Position::new(0, 1),
                &client_capabilities,
                None,
            )
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
