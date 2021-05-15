fn main() {
    println!("cargo:rerun-if-changed=src/citation/name/parser.lalrpop");

    #[cfg(feature = "citation")]
    lalrpop::process_root().unwrap();
}
