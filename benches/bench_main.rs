use criterion::{criterion_group, criterion_main, Criterion};
use texlab::latex;

static LATEX_PARSER_DATA: &str = include_str!("bench.tex");

fn latex_parser(criterion: &mut Criterion) {
    criterion.bench_function("Parser/LaTeX", |b| {
        b.iter(|| latex::parse(LATEX_PARSER_DATA))
    });
}

fn bibtex_parser(criterion: &mut Criterion) {
    drop(criterion);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = latex_parser, bibtex_parser
}

criterion_main! { benches }
