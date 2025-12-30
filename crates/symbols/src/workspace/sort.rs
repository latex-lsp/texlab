use base_db::{Document, Workspace};
use itertools::Itertools;
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering<'a> {
    inner: Vec<&'a Document>,
}

impl<'a> ProjectOrdering<'a> {
    pub fn get(&self, uri: &Url) -> usize {
        self.inner
            .iter()
            .position(|doc| doc.uri == *uri)
            .unwrap_or(std::usize::MAX)
    }
}

impl<'a> From<&'a Workspace> for ProjectOrdering<'a> {
    fn from(workspace: &'a Workspace) -> Self {
        let sorted_documents = || {
            workspace
                .iter()
                .sorted_by_key(|document| document.uri.as_str())
        };

        let inner = sorted_documents()
            .filter(|document| {
                let data = document.data.as_tex();
                data.is_some_and(|data| data.semantics.can_be_root)
            })
            .chain(sorted_documents())
            .flat_map(|document| {
                let graph = &workspace.graphs()[&document.uri];
                graph.preorder(workspace).rev().collect_vec()
            })
            .unique()
            .collect_vec();

        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use base_db::Owner;
    use distro::Language;
    use line_index::LineCol;

    use super::{ProjectOrdering, Url, Workspace};

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
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            b.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            c.clone(),
            r#"\documentclass{article}\begin{document}\include{b}\include{a}\end{document}"#
                .to_string(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        let ordering = ProjectOrdering::from(&workspace);
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
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            b.clone(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            c.clone(),
            r#"\begin{documnent}\include{b}\end{document}"#.to_string(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        let ordering = ProjectOrdering::from(&workspace);
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
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            b.clone(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            c.clone(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        let ordering = ProjectOrdering::from(&workspace);
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
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            b.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            c.clone(),
            String::new(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        workspace.open(
            d.clone(),
            r#"\begin{document}\include{c}\end{document}"#.to_string(),
            Language::Tex,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        let ordering = ProjectOrdering::from(&workspace);
        assert!(ordering.get(&b) < ordering.get(&a));
        assert!(ordering.get(&c) < ordering.get(&d));
    }
}
