mod compile;

pub use compile::*;
use std::path::Path;

pub fn build_test_code_header(file: &Path) -> Option<String> {
    let mut code = String::new();
    let name = file.file_stem()?.to_str()?;
    match file.extension()?.to_str()? {
        "cls" => code += &format!("\\documentclass{{{}}}\n", name),
        _ => {
            code += "\\documentclass{article}\n";
            code += &format!("\\usepackage{{{}}}\n", name);
        }
    };

    Some(code)
}
