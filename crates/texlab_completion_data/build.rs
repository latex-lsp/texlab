use std::fs;
use std::time::Duration;

fn main() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5 * 60))
        .build()
        .unwrap();

    let text = client.get("https://github.com/latex-lsp/latex-completion-data/releases/download/v19.07.1/completion.json")
        .send()
        .expect("Failed to download completion database")
        .text()
        .unwrap();

    fs::write("src/completion.json", text).expect("Failed to save completion database");
}
