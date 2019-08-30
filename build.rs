use std::process::Command;

fn main() {
    Command::new("node")
        .arg("src/citeproc/js/build.js")
        .output()
        .expect("Failed to bundle citeproc");
}
