use base_db::{semantics::Span, Document, DocumentData};
use rowan::{ast::AstNode, TextRange, TextSize};
use syntax::latex;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnvironmentMatch {
    pub name: Span,
    pub full_range: TextRange,
}

pub fn find_environments(document: &Document, offset: TextSize) -> Vec<EnvironmentMatch> {
    let DocumentData::Tex(data) = &document.data else { return Vec::new() };

    let root = latex::SyntaxNode::new_root(data.green.clone());

    let Some(token) = root.token_at_offset(offset).right_biased() else { return Vec::new() };

    let mut results = Vec::new();
    for environment in token
        .parent_ancestors()
        .filter_map(latex::Environment::cast)
    {
        let Some(name) = environment
            .begin()
            .and_then(|begin| begin.name())
            .and_then(|group| group.key())
            .map(|name| Span::from(&name)) else { continue };

        let full_range = latex::small_range(&environment);
        results.push(EnvironmentMatch { name, full_range });
    }

    results.reverse();
    results
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use test_utils::fixture::Fixture;

    use crate::find_environments;

    #[test]
    fn test() {
        let fixture = Fixture::parse(
            r#"
%! main.tex
\begin{a}
  \begin{b}
    \begin{c}
      |
    \end{c}
  \end{b}
  \begin{d}
  \end{d}
\end{a}"#,
        );

        let workspace = fixture.workspace;
        let document = workspace.iter().next().unwrap();
        let results = find_environments(&document, fixture.documents[0].cursor.unwrap());
        assert_debug_snapshot!(results);
    }
}
