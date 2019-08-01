use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

fn sh(command: &'static str, directory: &Path) {
    let (executable, args) = if cfg!(windows) {
        ("cmd", vec!["/C", command])
    } else {
        ("sh", vec!["-c", command])
    };

    Command::new(executable)
        .args(args)
        .current_dir(directory)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect(&format!("Failed to execute \"{}\"", command));
}

fn bundle_citeproc() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates")
        .join("citeproc")
        .join("script");

    sh("npm ci", &directory);
    sh("npm run dist", &directory);
}

fn download_completion_database() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5 * 60))
        .build()
        .unwrap();

    let text = client.get("https://github.com/latex-lsp/latex-completion-data/releases/download/v19.07.1/completion.json")
        .send()
        .expect("Failed to download completion database")
        .text()
        .unwrap();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates")
        .join("texlab_completion_data")
        .join("completion.json");
        
    fs::write(path, text).expect("Failed to save completion database");
}

fn main() {
    bundle_citeproc();
    download_completion_database();
}
