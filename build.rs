fn main() {
    println!("cargo:rerun-if-changed=src/citeproc/name/parser.lalrpop");
    lalrpop::process_root().unwrap();
}
