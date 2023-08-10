use base_db::{util::LineCol, Owner, Workspace};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use distro::Language;
use lsp_types::{ClientCapabilities, Position, Url};
use parser::{parse_latex, SyntaxConfig};

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
            LineCol { line: 0, col: 0 },
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

    c.bench_function("BibTeX/Cite", |b| {
        let uri1 = Url::parse("http://example.com/texlab.tex").unwrap();
        let uri2 = Url::parse("http://example.com/rht.bib").unwrap();
        let text1 = r#"\cite{a}\addbibresource{rht.bib}"#.to_string();
        let text2 = include_str!("/home/paddy/texlab-testing/rht.bib").to_string();
        let mut workspace = Workspace::default();
        workspace.open(
            uri1.clone(),
            text1,
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            uri2.clone(),
            text2,
            Language::Bib,
            Owner::Client,
            LineCol { line: 0, col: 6 },
        );

        let client_capabilities = ClientCapabilities::default();

        b.iter(|| {
            texlab::features::completion::complete(
                &workspace,
                &uri1,
                Position::new(0, 7),
                &client_capabilities,
                None,
            )
        });
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
