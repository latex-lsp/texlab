use criterion::{criterion_group, Criterion};
use futures::executor::block_on;
use lsp_types::*;
use texlab::test_scenario::{TestScenario, DEFAULT_CAPABILITIES};

fn initialize(name: &'static str) -> TestScenario {
    let scenario = block_on(TestScenario::new("completion/bench", &DEFAULT_CAPABILITIES));
    block_on(scenario.open(name));
    scenario
}

fn run(scenario: &TestScenario, name: &str, position: Position) {
    let uri = scenario.uri(name);
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier::new(uri.into()),
            position,
        },
        context: None,
    };
    block_on(scenario.server.completion(params)).unwrap();
}

fn criterion_benchmark(criterion: &mut Criterion) {
    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX word", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(5, 0));
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX command", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(6, 1));
        })
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX argument symbol", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(7, 8));
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX environment", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(8, 7));
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX class import", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(9, 15));
        });
    });

    let scenario = initialize("foo.tex");
    criterion.bench_function("LaTeX package import", move |b| {
        b.iter(|| {
            run(&scenario, "foo.tex", Position::new(10, 12));
        });
    });

    let scenario = initialize("foo.bib");
    criterion.bench_function("BibTeX type", move |b| {
        b.iter(|| {
            run(&scenario, "foo.bib", Position::new(0, 1));
        });
    });

    let scenario = initialize("foo.bib");
    criterion.bench_function("BibTeX field", move |b| {
        b.iter(|| {
            run(&scenario, "foo.bib", Position::new(3, 5));
        });
    });

    let scenario = initialize("foo.bib");
    criterion.bench_function("BibTeX command", move |b| {
        b.iter(|| {
            run(&scenario, "foo.bib", Position::new(7, 14));
        });
    });
}

criterion_group!(benches, criterion_benchmark);

#[tokio::main]
async fn main() {
    benches();
}
