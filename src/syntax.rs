use cstree::TextRange;

pub mod bibtex;
pub mod build_log;
pub mod latex;

pub trait CstNode<'a> {
    type Lang: cstree::Language;

    fn cast(node: &'a cstree::ResolvedNode<Self::Lang>) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &'a cstree::ResolvedNode<Self::Lang>;

    fn small_range(&self) -> TextRange;
}
