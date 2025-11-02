use criterion::{Criterion, black_box, criterion_group, criterion_main};
use parser::{SyntaxConfig, parse_latex};

const CODE: &str = include_str!("../../../texlab.tex");

fn criterion_benchmark(c: &mut Criterion) {
    let config = SyntaxConfig::default();
    c.bench_function("LaTeX/Parser", |b| {
        b.iter(|| parse_latex(black_box(CODE), &config));
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
