use criterion::criterion_group;
use criterion::{BenchmarkId, Criterion};
use futures::executor::block_on;
use indoc::indoc;
use std::time::Duration;
use texlab::{completion::CompletionProvider, feature::FeatureTester};

fn criterion_benchmark(criterion: &mut Criterion) {
    criterion.bench_with_input(
        BenchmarkId::new("Completion", "LaTeX Word"),
        &LATEX_CODE,
        |b, code| {
            b.iter(|| {
                block_on(async {
                    FeatureTester::new()
                        .file("main.tex", *code)
                        .main("main.tex")
                        .position(0, 0)
                        .test_completion(CompletionProvider::new())
                        .await
                });
            });
        },
    );

    criterion.bench_with_input(
        BenchmarkId::new("Completion", "LaTeX Command"),
        &LATEX_CODE,
        |b, code| {
            b.iter(|| {
                block_on(async {
                    FeatureTester::new()
                        .file("main.tex", *code)
                        .main("main.tex")
                        .position(0, 1)
                        .test_completion(CompletionProvider::new())
                        .await
                });
            });
        },
    );

    criterion.bench_with_input(
        BenchmarkId::new("Completion", "LaTeX Environment"),
        &LATEX_CODE,
        |b, code| {
            b.iter(|| {
                block_on(async {
                    FeatureTester::new()
                        .file("main.tex", *code)
                        .main("main.tex")
                        .position(9, 9)
                        .test_completion(CompletionProvider::new())
                        .await
                });
            })
        },
    );

    criterion.bench_with_input(
        BenchmarkId::new("Completion", "LaTeX Label"),
        &LATEX_CODE,
        |b, code| {
            b.iter(|| {
                block_on(async {
                    FeatureTester::new()
                        .file("main.tex", *code)
                        .main("main.tex")
                        .position(15, 7)
                        .test_completion(CompletionProvider::new())
                        .await
                });
            })
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(60));
    targets = criterion_benchmark
}

static LATEX_CODE: &str = indoc!(
    r#"
        \documentclass{article}
        \usepackage{amsmath}
        \usepackage{lipsum}
        \usepackage{geometry}
        \usepackage[utf8]{inputenc}
        \newcommand{\foo}{foo}
        \DeclareMathOperator{\bar}{bar}
        \include{child1}
        \input{child2.tex}
        \begin{document}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \begin{equation*}\label{eq:foo}
            e^{i \pi} + 1 = 0
        \end{equation*}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \ref{eq:foo}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \section{Foo}\label{sec:foo}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \subsection{Bar}\label{sec:bar}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \include{foo}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \input{bar.tex}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \cite{foo, bar, baz}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \nocite{*}
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
        \end{document}
    "#
);
