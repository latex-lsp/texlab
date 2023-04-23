pub mod auxiliary;
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
        Span {
            text: key.to_string(),
            range: syntax::latex::small_range(key),
        }
    }
}
