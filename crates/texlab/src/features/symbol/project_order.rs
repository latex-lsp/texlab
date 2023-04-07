use itertools::Itertools;
use lsp_types::Url;

use crate::{
    db::{dependency_graph, Document, Workspace},
    Db,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProjectOrdering {
    ordering: Vec<Document>,
}

impl ProjectOrdering {
    pub fn new(db: &dyn Db) -> Self {
        let workspace = Workspace::get(db);

        let ordering: Vec<_> = workspace
            .index_files(db)
            .chain(workspace.documents(db).iter().copied())
            .flat_map(|document| {
                dependency_graph(db, document)
                    .preorder()
                    .rev()
                    .collect_vec()
            })
            .unique()
            .collect();

        Self { ordering }
    }

    pub fn get(&self, db: &dyn Db, uri: &Url) -> usize {
        self.ordering
            .iter()
            .position(|doc| doc.location(db).uri(db) == uri)
            .unwrap_or(std::usize::MAX)
    }
}

#[cfg(test)]
mod tests {
    use distro::Language;

    use crate::{db::Owner, Database};

    use super::*;

    #[test]
    fn test_no_cycles() {
        let mut db = Database::default();
        let workspace = Workspace::get(&db);

        let a = workspace.open(
            &mut db,
            Url::parse("http://example.com/a.tex").unwrap(),
            String::new(),
            Language::Tex,
            Owner::Client,
        );

        let b = workspace.open(
            &mut db,
            Url::parse("http://example.com/b.tex").unwrap(),
            String::new(),
            Language::Tex,
            Owner::Client,
        );

        let c = workspace.open(
            &mut db,
            Url::parse("http://example.com/c.tex").unwrap(),
            r#"\documentclass{article}\include{b}\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        let ordering = ProjectOrdering::new(&db);

        assert_eq!(ordering.get(&db, a.location(&db).uri(&db)), 0);
        assert_eq!(ordering.get(&db, b.location(&db).uri(&db)), 1);
        assert_eq!(ordering.get(&db, c.location(&db).uri(&db)), 2);
    }

    #[test]
    fn test_two_layers() {
        let mut db = Database::default();
        let workspace = Workspace::get(&db);

        let a = workspace.open(
            &mut db,
            Url::parse("http://example.com/a.tex").unwrap(),
            String::new(),
            Language::Tex,
            Owner::Client,
        );

        let b = workspace.open(
            &mut db,
            Url::parse("http://example.com/b.tex").unwrap(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        let c = workspace.open(
            &mut db,
            Url::parse("http://example.com/c.tex").unwrap(),
            r#"\documentclass{article}\include{b}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        let ordering = ProjectOrdering::new(&db);

        assert_eq!(ordering.get(&db, a.location(&db).uri(&db)), 0);
        assert_eq!(ordering.get(&db, b.location(&db).uri(&db)), 1);
        assert_eq!(ordering.get(&db, c.location(&db).uri(&db)), 2);
    }

    #[test]
    fn test_cycles() {
        let mut db = Database::default();
        let workspace = Workspace::get(&db);

        let a = workspace.open(
            &mut db,
            Url::parse("http://example.com/a.tex").unwrap(),
            r#"\documentclass{article}\include{b}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        workspace.open(
            &mut db,
            Url::parse("http://example.com/b.tex").unwrap(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        workspace.open(
            &mut db,
            Url::parse("http://example.com/c.tex").unwrap(),
            r#"\include{a}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        let ordering = ProjectOrdering::new(&db);
        assert_ne!(ordering.get(&db, a.location(&db).uri(&db)), 0);
    }

    #[test]
    fn test_multiple_roots() {
        let mut db = Database::default();
        let workspace = Workspace::get(&db);

        let a = workspace.open(
            &mut db,
            Url::parse("http://example.com/a.tex").unwrap(),
            r#"\documentclass{article}\include{b}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        let b = workspace.open(
            &mut db,
            Url::parse("http://example.com/b.tex").unwrap(),
            String::new(),
            Language::Tex,
            Owner::Client,
        );

        let c = workspace.open(
            &mut db,
            Url::parse("http://example.com/c.tex").unwrap(),
            String::new(),
            Language::Tex,
            Owner::Client,
        );

        let d = workspace.open(
            &mut db,
            Url::parse("http://example.com/d.tex").unwrap(),
            r#"\documentclass{article}\include{c}"#.to_string(),
            Language::Tex,
            Owner::Client,
        );

        let ordering = ProjectOrdering::new(&db);
        assert!(
            ordering.get(&db, b.location(&db).uri(&db))
                < ordering.get(&db, a.location(&db).uri(&db))
        );
        assert!(
            ordering.get(&db, c.location(&db).uri(&db))
                < ordering.get(&db, d.location(&db).uri(&db))
        );
    }
}
