fn main() {
    println!("cargo:rerun-if-changed=src/name/parser.lalrpop");
    lalrpop::process_root().unwrap();
}
