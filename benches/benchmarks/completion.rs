use super::test_data::TEST_LATEX;
use criterion::{BenchmarkId, Criterion};
use futures::executor::block_on;
use std::time::Duration;
use texlab_completion::CompletionProvider;
use texlab_feature::FeatureTester;

fn criterion_benchmark(criterion: &mut Criterion) {
    criterion.bench_with_input(
        BenchmarkId::new("Completion", "LaTeX Command"),
        &TEST_LATEX,
        |b, code| {
            b.iter(|| {
                let items = block_on(async {
                    FeatureTester::new()
                        .file("main.tex", code.as_str())
                        .main("main.tex")
                        .position(0, 1)
                        .test_completion(CompletionProvider::new())
                        .await
                });
                assert!(!items.is_empty());
            });
        },
    );

    criterion.bench_with_input(
        BenchmarkId::new("Completion", "LaTeX Environment"),
        &TEST_LATEX,
        |b, code| {
            b.iter(|| {
                let items = block_on(async {
                    FeatureTester::new()
                        .file("main.tex", code.as_str())
                        .main("main.tex")
                        .position(9, 9)
                        .test_completion(CompletionProvider::new())
                        .await
                });
                assert!(!items.is_empty());
            })
        },
    );
}

pub fn benches() {
    let mut criterion = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .configure_from_args();

    criterion_benchmark(&mut criterion);
}
