use std::{sync::Arc, usize};

use lsp_types::Url;
use rustc_hash::FxHashSet;

use crate::Workspace;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering {
    ordering: Vec<Arc<Url>>,
}

impl ProjectOrdering {
    pub fn get(&self, uri: &Url) -> usize {
        self.ordering
            .iter()
            .position(|u| u.as_ref() == uri)
            .unwrap_or(usize::MAX)
    }
}

impl From<&Workspace> for ProjectOrdering {
    fn from(workspace: &Workspace) -> Self {
        let mut ordering = Vec::new();
        let mut visited = FxHashSet::default();
        for root in workspace.project_roots().chain(workspace.iter()) {
            ordering.extend(
                workspace
                    .project_files(&root)
                    .into_iter()
                    .map(|doc| Arc::clone(doc.uri()))
                    .filter(|uri| visited.insert(Arc::clone(uri))),
            );
        }

        Self { ordering }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;

    use crate::DocumentLanguage;

    use super::*;

    #[test]
    fn test_no_cycles() -> Result<()> {
        let mut workspace = Workspace::default();

        let a = workspace.open(
            Arc::new(Url::parse("http://example.com/a.tex")?),
            Arc::new(String::new()),
            DocumentLanguage::Latex,
        )?;

        let b = workspace.open(
            Arc::new(Url::parse("http://example.com/b.tex")?),
            Arc::new(String::new()),
            DocumentLanguage::Latex,
        )?;

        let c = workspace.open(
            Arc::new(Url::parse("http://example.com/c.tex")?),
            Arc::new(r#"\documentclass{article}\include{b}\include{a}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let ordering = ProjectOrdering::from(&workspace);
        assert_eq!(ordering.get(a.uri()), 2);
        assert_eq!(ordering.get(b.uri()), 1);
        assert_eq!(ordering.get(c.uri()), 0);
        Ok(())
    }

    #[test]
    fn test_no_cycles_reverse() -> Result<()> {
        let mut workspace = Workspace::default();

        let a = workspace.open(
            Arc::new(Url::parse("http://example.com/a.tex")?),
            Arc::new(r#"\documentclass{article}\include{b}\include{c}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let b = workspace.open(
            Arc::new(Url::parse("http://example.com/b.tex")?),
            Arc::new(String::new()),
            DocumentLanguage::Latex,
        )?;

        let c = workspace.open(
            Arc::new(Url::parse("http://example.com/c.tex")?),
            Arc::new(String::new()),
            DocumentLanguage::Latex,
        )?;

        let ordering = ProjectOrdering::from(&workspace);
        assert_eq!(ordering.get(a.uri()), 0);
        assert_eq!(ordering.get(b.uri()), 1);
        assert_eq!(ordering.get(c.uri()), 2);
        Ok(())
    }

    #[test]
    fn test_cycles() -> Result<()> {
        let mut workspace = Workspace::default();

        let a = workspace.open(
            Arc::new(Url::parse("http://example.com/a.tex")?),
            Arc::new(r#"\include{b}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let b = workspace.open(
            Arc::new(Url::parse("http://example.com/b.tex")?),
            Arc::new(r#"\include{a}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let c = workspace.open(
            Arc::new(Url::parse("http://example.com/c.tex")?),
            Arc::new(r#"\include{a}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let ordering = ProjectOrdering::from(&workspace);

        assert_eq!(ordering.get(a.uri()), 1);
        assert_eq!(ordering.get(b.uri()), 2);
        assert_eq!(ordering.get(c.uri()), 0);
        Ok(())
    }

    #[test]
    fn test_multiple_roots() -> Result<()> {
        let mut workspace = Workspace::default();

        let a = workspace.open(
            Arc::new(Url::parse("http://example.com/a.tex")?),
            Arc::new(r#"\include{b}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let b = workspace.open(
            Arc::new(Url::parse("http://example.com/b.tex")?),
            Arc::new(r#""#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let c = workspace.open(
            Arc::new(Url::parse("http://example.com/c.tex")?),
            Arc::new(r#""#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let d = workspace.open(
            Arc::new(Url::parse("http://example.com/d.tex")?),
            Arc::new(r#"\include{c}"#.to_string()),
            DocumentLanguage::Latex,
        )?;

        let ordering = ProjectOrdering::from(&workspace);

        assert!(ordering.get(a.uri()) < ordering.get(b.uri()));
        assert!(ordering.get(d.uri()) < ordering.get(c.uri()));
        Ok(())
    }
}
