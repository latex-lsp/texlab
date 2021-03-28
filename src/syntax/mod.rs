pub mod bibtex;
pub mod latex;

pub trait AstNode {
    type Lang: rowan::Language;

    fn cast(node: rowan::SyntaxNode<Self::Lang>) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &rowan::SyntaxNode<Self::Lang>;
}
