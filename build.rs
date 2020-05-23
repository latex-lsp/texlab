fn main() {
    println!("cargo:rerun-if-changed=src/citeproc/name/parser.lalrpop");

    #[cfg(feature = "citation")]
    lalrpop::process_root().unwrap();
}
