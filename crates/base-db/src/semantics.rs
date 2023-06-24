pub mod auxiliary;
pub mod bib;
pub mod tex;

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Span {
    pub text: String,
    pub range: rowan::TextRange,
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

impl From<&syntax::bibtex::SyntaxToken> for Span {
    fn from(token: &syntax::bibtex::SyntaxToken) -> Self {
        Self {
            text: token.text().into(),
            range: token.text_range(),
        }
    }
}
