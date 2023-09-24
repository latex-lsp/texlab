pub mod auxiliary;
pub mod bib;
pub mod tex;

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Span {
    pub text: String,
    pub range: rowan::TextRange,
}

impl Span {
    pub fn new(text: String, range: rowan::TextRange) -> Self {
        Self { text, range }
    }

    pub fn empty(offset: rowan::TextSize) -> Self {
        Self {
            text: String::new(),
            range: rowan::TextRange::empty(offset),
        }
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Span")
            .field(&self.text)
            .field(&self.range)
            .finish()
    }
}

impl From<&syntax::latex::Key> for Span {
    fn from(key: &syntax::latex::Key) -> Self {
        Self {
            text: key.to_string(),
            range: syntax::latex::small_range(key),
        }
    }
}

impl<L: rowan::Language> From<&rowan::SyntaxToken<L>> for Span {
    fn from(token: &rowan::SyntaxToken<L>) -> Self {
        Self {
            text: token.text().into(),
            range: token.text_range(),
        }
    }
}
