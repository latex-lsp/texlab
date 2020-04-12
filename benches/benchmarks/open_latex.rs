use super::test_data::TEST_LATEX;
use criterion::Criterion;
use std::env;
use texlab_protocol::{Options, Uri};
use texlab_syntax::latex;
use texlab_tex::Resolver;

fn criterion_benchmark(criterion: &mut Criterion) {
    let uri = Uri::parse("file:///home/user/main.tex").unwrap();
    let resolver = Resolver::default();
    let options = Options::default();
    let cwd = env::current_dir().unwrap();

    criterion.bench_function("LaTeX Analysis", |b| {
        b.iter(|| {
            latex::open(latex::OpenParams {
                text: &TEST_LATEX,
                uri: &uri,
                resolver: &resolver,
                options: &options,
                current_dir: &cwd,
            })
        })
    });
}

pub fn benches() {
    let mut criterion = Criterion::default().configure_from_args();
    criterion_benchmark(&mut criterion);
}
