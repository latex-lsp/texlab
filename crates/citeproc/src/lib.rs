mod driver;
mod entry;
mod output;

use syntax::bibtex;
use unicode_normalization::UnicodeNormalization;

use self::{driver::Driver, output::Inline};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Mode {
    Detailed,
    Overview,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Detailed
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub mode: Mode,
}

#[must_use]
pub fn render(entry: &bibtex::Entry, options: &Options) -> Option<String> {
    let mut output = String::new();
    let mut driver = Driver::new(options);
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
