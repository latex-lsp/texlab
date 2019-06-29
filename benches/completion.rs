#![feature(async_await)]

use criterion::{criterion_group, Criterion};
use futures::executor::block_on;
use lsp_types::*;
use texlab::scenario::{Scenario, FULL_CAPABILITIES};

fn initialize(name: &'static str) -> Scenario {
    let scenario = block_on(Scenario::new("completion/bench", &FULL_CAPABILITIES));
    block_on(scenario.open(name));
    block_on(scenario.server.stop_scanning());
    scenario
}

fn run(scenario: &Scenario, name: &str, position: Position, has_items: bool) {
    let uri = scenario.uri(name);
    let params = CompletionParams {
        text_document: TextDocumentIdentifier::new(uri),
        position,
        context: None,
    };
    let items = block_on(scenario.server.completion(params)).unwrap();
    assert_eq!(items.items.len() > 0, has_items);
}

fn criterion_benchmark(criterion: &mut Criterion) {
    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX word", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(5, 0), false);
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX command", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(6, 1), true);
        })
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX argument symbol", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(7, 8), true);
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX environment", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(8, 7), true);
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX class import", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(9, 15), true);
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX package import", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(10, 12), true);
        });
    });

    let scenario = initialize("foo.bib");
    criterion.bench_function("BibTeX type", move |b| {
        b.iter(|| {
            run(&scenario, "foo.bib", Position::new(0, 1), true);
        });
    });

    let scenario = initialize("foo.bib");
    criterion.bench_function("BibTeX field", move |b| {
        b.iter(|| {
            run(&scenario, "foo.bib", Position::new(3, 5), true);
        });
    });

    let scenario = initialize("foo.bib");
    criterion.bench_function("BibTeX command", move |b| {
        b.iter(|| {
            run(&scenario, "foo.bib", Position::new(7, 14), true);
        });
    });
}

criterion_group!(benches, criterion_benchmark);

#[runtime::main(runtime_tokio::Tokio)]
async fn main() {
    benches();
}
