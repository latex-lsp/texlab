use criterion::{black_box, criterion_group, criterion_main, Criterion};
use texlab::syntax::latex;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| latex::parse(black_box(include_str!("../texlab.tex"))));
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
