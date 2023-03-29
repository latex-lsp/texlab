mod driver;
mod entry;
pub(crate) mod field;
mod output;

use syntax::bibtex;
use unicode_normalization::UnicodeNormalization;

use self::{driver::Driver, output::Inline};

#[must_use]
pub fn render(entry: &bibtex::Entry) -> Option<String> {
    let mut output = String::new();
    let mut driver = Driver::default();
    driver.process(entry);
    driver.finish().for_each(|(inline, punct)| {
        let text = match inline {
            Inline::Regular(text) => text,
            Inline::Italic(text) => format!("*{text}*"),
            Inline::Quoted(text) => format!("\"{text}\""),
            Inline::Link { url, alt } => format!("[{alt}]({url})"),
        };
        output.push_str(&text);
        output.push_str(punct.as_str());
    });

    if output.is_empty() {
        None
    } else {
        output.push('.');
        Some(output.nfc().collect())
    }
}

#[cfg(test)]
mod tests;
