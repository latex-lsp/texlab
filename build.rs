use std::fs;

fn main() {
    let text = reqwest::get("https://github.com/latex-lsp/latex-completion-data/releases/download/v19.07.1/completion.json")
        .expect("Failed to download completion database")
        .text()
        .unwrap();
    fs::write("src/data/completion.json", text).expect("Failed to save completion database");
}
