use base_db::{util::LineCol, FeatureParams, Owner, Workspace};
use completion::CompletionParams;
use criterion::{criterion_group, criterion_main, Criterion};
use distro::Language;
use rowan::TextSize;
use url::Url;

const CODE: &str = include_str!("../../../texlab.tex");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Command", |b| {
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

        let feature = FeatureParams::new(&workspace, workspace.lookup(&uri).unwrap());
        let params = CompletionParams {
            feature,
            offset: TextSize::from(1),
        };

        b.iter(|| completion::complete(&params));
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
