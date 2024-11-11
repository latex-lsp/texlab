mod builder;
pub mod matchers;
mod patterns;

pub use builder::*;
pub use patterns::*;

pub struct ProviderContext<'a, 'b> {
    pub builder: &'b mut CompletionBuilder<'a>,
    pub params: &'a crate::CompletionParams<'a>,
    pub cursor: base_db::semantics::Span,
}
