use std::borrow::Cow;

use itertools::Itertools;
use lsp_types::Url;
use rustc_hash::FxHashSet;
use typed_builder::TypedBuilder;

use crate::syntax::latex::ExplicitLink;

use super::{Document, DocumentData, Workspace};

#[derive(Debug, Clone)]
pub struct Group<'a> {
    pub from: &'a Document,
    pub to: Option<&'a Document>,
    pub link: Option<&'a ExplicitLink>,
    pub locations: Vec<Url>,
}

#[derive(Debug, Clone)]
pub struct Graph<'a> {
    pub start: &'a Document,
    pub working_dir: &'a Url,
    pub groups: Vec<Group<'a>>,
}

impl<'a> Graph<'a> {
    pub fn preorder(&self) -> impl Iterator<Item = &'a Document> + '_ {
        std::iter::once(self.start)
            .chain(self.groups.iter().filter_map(|group| group.to))
            .unique_by(|document| &document.uri)
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Analyzer<'a> {
    workspace: &'a Workspace,
    start: &'a Document,
    working_dir: &'a Url,
    output_dir: &'a str,
}

impl<'a> Analyzer<'a> {
    pub fn run(self) -> Graph<'a> {
        let mut groups = Vec::new();
        let mut stack = vec![(self.start, Cow::Borrowed(self.working_dir))];
        let mut visited = FxHashSet::default();

        while let Some((parent, working_dir)) = stack.pop() {
            let data = match &parent.data {
                DocumentData::Tex(data) => data,
                _ => continue,
            };

            for link in &data.extras.explicit_links {
                let locations = link
                    .full_paths()
                    .flat_map(|path| working_dir.join(&path))
                    .collect::<Vec<_>>();

                let child = locations.iter().find_map(|uri| self.workspace.get(uri));

                groups.push(Group {
                    from: parent,
                    to: child,
                    link: Some(link),
                    locations,
                });

                if let Some(child) = child {
                    if visited.insert(&child.uri) {
                        let new_working_dir = link
                            .working_dir
                            .as_deref()
                            .and_then(|path| working_dir.join(path).ok())
                            .map_or_else(|| working_dir.clone(), Cow::Owned);

                        stack.push((child, new_working_dir));
                    }
                }
            }

            groups.extend(self.artifact(parent, &working_dir, "aux"));
            groups.extend(self.artifact(parent, &working_dir, "log"));
        }

        Graph {
            start: self.start,
            working_dir: self.working_dir,
            groups,
        }
    }

    fn artifact(
        &self,
        parent: &'a Document,
        working_dir: &Url,
        extension: &str,
    ) -> Option<Group<'a>> {
        let name = parent.uri.path_segments()?.last()?;
        let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
        let location = working_dir
            .join(self.output_dir)
            .ok()?
            .join(&format!("{stem}.{extension}"))
            .ok()?;

        Some(Group {
            from: parent,
            to: self.workspace.get(&location),
            link: None,
            locations: vec![location],
        })
    }
}
