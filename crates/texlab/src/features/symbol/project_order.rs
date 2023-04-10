use base_db::{graph, Document, Workspace};
use itertools::Itertools;
use lsp_types::Url;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering<'a> {
    ordering: Vec<&'a Document>,
}

impl<'a> ProjectOrdering<'a> {
    pub fn new(workspace: &'a Workspace) -> Self {
        let ordering: Vec<_> = workspace
            .iter()
            .filter(|document| {
                document
                    .data
                    .as_tex()
                    .map_or(false, |data| data.semantics.can_be_root)
            })
            .chain(workspace.iter())
            .flat_map(|document| {
                graph::Graph::new(workspace, document)
                    .preorder()
                    .rev()
                    .collect_vec()
            })
            .unique()
            .collect();

        Self { ordering }
    }

    pub fn get(&self, uri: &Url) -> usize {
        self.ordering
            .iter()
            .position(|doc| doc.uri == *uri)
            .unwrap_or(std::usize::MAX)
    }
}

#[cfg(test)]
mod tests {
    use base_db::Owner;
    use distro::Language;
    use rowan::TextSize;

    use super::*;

    #[test]
    fn test_no_cycles() {
        let mut workspace = Workspace::default();

        let a = Url::parse("http://example.com/a.tex").unwrap();
        let b = Url::parse("http://example.com/b.tex").unwrap();
        let c = Url::parse("http://example.com/c.tex").unwrap();

        workspace.open(
            a.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            b.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            c.clone(),
            r#"\documentclass{article}\include{b}\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        let ordering = ProjectOrdering::new(&workspace);
        assert_eq!(ordering.get(&a), 0);
        assert_eq!(ordering.get(&b), 1);
        assert_eq!(ordering.get(&c), 2);
    }

    #[test]
    fn test_two_layers() {
        let mut workspace = Workspace::default();

        let a = Url::parse("http://example.com/a.tex").unwrap();
        let b = Url::parse("http://example.com/b.tex").unwrap();
        let c = Url::parse("http://example.com/c.tex").unwrap();

        workspace.open(
            a.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );
        workspace.open(
            b.clone(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );
        workspace.open(
            c.clone(),
            r#"\begin{documnent}\include{b}\end{document}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        let ordering = ProjectOrdering::new(&workspace);
        assert_eq!(ordering.get(&a), 0);
        assert_eq!(ordering.get(&b), 1);
        assert_eq!(ordering.get(&c), 2);
    }

    #[test]
    fn test_cycles() {
        let mut workspace = Workspace::default();

        let a = Url::parse("http://example.com/a.tex").unwrap();
        let b = Url::parse("http://example.com/b.tex").unwrap();
        let c = Url::parse("http://example.com/c.tex").unwrap();
        workspace.open(
            a.clone(),
            r#"\begin{document}\include{b}\end{document}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            b.clone(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            c.clone(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        let ordering = ProjectOrdering::new(&workspace);
        assert_ne!(ordering.get(&a), 0);
    }

    #[test]
    fn test_multiple_roots() {
        let mut workspace = Workspace::default();

        let a = Url::parse("http://example.com/a.tex").unwrap();
        let b = Url::parse("http://example.com/b.tex").unwrap();
        let c = Url::parse("http://example.com/c.tex").unwrap();
        let d = Url::parse("http://example.com/d.tex").unwrap();

        workspace.open(
            a.clone(),
            r#"\begin{document}\include{b}\end{document}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            b.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            c.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        workspace.open(
            d.clone(),
            r#"\begin{document}\include{c}\end{document}"#.to_string(),
            Language::Tex,
            Owner::Client,
            TextSize::default(),
        );

        let ordering = ProjectOrdering::new(&workspace);
        assert!(ordering.get(&b) < ordering.get(&a));
        assert!(ordering.get(&c) < ordering.get(&d));
    }
}
