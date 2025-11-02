use base_db::semantics::Span;
use completion_data::included_packages;
use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    CompletionItem, CompletionItemData, CompletionParams, EnvironmentData,
    util::{CompletionBuilder, ProviderContext, find_curly_group_word},
};

pub fn complete_environments<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let (cursor, group) = find_environment(params)?;

    let begin = group
        .syntax()
        .parent()
        .and_then(|node| node.parent())
        .and_then(latex::Environment::cast)
        .and_then(|env| env.begin())
        .and_then(|begin| begin.name())
        .and_then(|name| name.key())
        .map_or_else(String::new, |name| name.to_string());

    let mut proc = Processor {
        inner: ProviderContext {
            builder,
            params,
            cursor,
        },
        begin,
    };

    proc.add_library();
    proc.add_theorem();
    proc.add_user();
    Some(())
}

struct Processor<'a, 'b> {
    inner: ProviderContext<'a, 'b>,
    begin: String,
}

impl<'a, 'b> Processor<'a, 'b> {
    fn add_library(&mut self) {
        for package in included_packages(&self.inner.params.feature) {
            let envs_with_score = package.environments.iter().filter_map(|env| {
                let matcher = &self.inner.builder.matcher;
                let score = matcher.score(env, &self.inner.cursor.text)?;
                Some((*env, score))
            });

            for (name, score) in envs_with_score {
                let data = CompletionItemData::Environment(EnvironmentData {
                    name,
                    package: Some(package),
                });

                self.inner.builder.items.push(CompletionItem {
                    score,
                    data,
                    range: self.inner.cursor.range,
                    preselect: name == self.begin,
                });
            }
        }
    }

    fn add_theorem(&mut self) {
        let documents = self.inner.params.feature.project.documents.iter();
        for theorem in documents
            .filter_map(|document| document.data.as_tex())
            .flat_map(|data| data.semantics.theorem_definitions.iter())
        {
            let matcher = &self.inner.builder.matcher;
            let name = theorem.name.text.as_str();
            if let Some(score) = matcher.score(name, &self.inner.cursor.text) {
                let data = CompletionItemData::Environment(EnvironmentData {
                    name,
                    package: None,
                });

                self.inner.builder.items.push(CompletionItem {
                    score,
                    data,
                    range: self.inner.cursor.range,
                    preselect: name == self.begin,
                });
            }
        }
    }

    fn add_user(&mut self) {
        let documents = self.inner.params.feature.project.documents.iter();
        for data in documents.filter_map(|document| document.data.as_tex()) {
            let envs = data
                .semantics
                .environments
                .iter()
                .filter(|name| name.range != self.inner.cursor.range);

            let envs_with_score = envs.filter_map(|env| {
                let matcher = &self.inner.builder.matcher;
                let score = matcher.score(&env.text, &self.inner.cursor.text)?;
                Some((&env.text, score))
            });

            for (name, score) in envs_with_score {
                let data = CompletionItemData::Environment(EnvironmentData {
                    name,
                    package: None,
                });

                self.inner.builder.items.push(CompletionItem {
                    score,
                    data,
                    range: self.inner.cursor.range,
                    preselect: name == &self.begin,
                });
            }
        }
    }
}

fn find_environment(params: &CompletionParams) -> Option<(Span, latex::CurlyGroupWord)> {
    let (span, group) = find_curly_group_word(params)?;
    let parent = group.syntax().parent()?;
    if matches!(parent.kind(), latex::BEGIN | latex::END) {
        Some((span, group))
    } else {
        None
    }
}
