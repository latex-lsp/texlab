use super::test_data::TEST_BIBTEX;
use criterion::Criterion;
use texlab_syntax::bibtex;

fn criterion_benchmark(criterion: &mut Criterion) {
    criterion.bench_function("BibTeX Parser", |b| b.iter(|| bibtex::open(&TEST_BIBTEX)));
}

pub fn benches() {
    let mut criterion = Criterion::default().configure_from_args();
    criterion_benchmark(&mut criterion);
}
