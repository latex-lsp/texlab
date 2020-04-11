mod benchmarks;

use criterion::Criterion;

fn main() {
    benchmarks::completion::benches();
    benchmarks::open_bibtex::benches();
    benchmarks::open_latex::benches();
    Criterion::default().configure_from_args().final_summary();
}
