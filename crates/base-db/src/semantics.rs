pub mod aux;
pub mod tex;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Span {
    pub text: String,
    pub range: rowan::TextRange,
}

impl From<&syntax::latex::Key> for Span {
    fn from(key: &syntax::latex::Key) -> Self {
        Span {
            text: key.to_string(),
            range: syntax::latex::small_range(key),
        }
    }
}
