pub mod bibtex;
pub mod latex;

pub trait CstNode<'a, D>
where
    D: 'static,
{
    type Lang: cstree::Language;

    fn cast(node: &'a cstree::ResolvedNode<Self::Lang, D>) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &'a cstree::ResolvedNode<Self::Lang, D>;
}
