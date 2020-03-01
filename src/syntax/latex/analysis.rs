use crate::{
    protocol::{Options, Uri},
    syntax::{lang_data::*, latex::ast::*},
    tex::Resolver,
};
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SymbolTableParams<'a> {
    pub tree: Tree,
    pub uri: &'a Uri,
    pub resolver: &'a Resolver,
    pub options: &'a Options,
    pub cwd: &'a Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    pub tree: Tree,
    pub commands: Vec<NodeIndex>,
    pub environments: Vec<Environment>,
    pub includes: Vec<Include>,
    pub components: Vec<String>,
}

impl SymbolTable {
    pub fn new(params: SymbolTableParams) -> Self {
        let SymbolTableParams {
            tree,
            uri,
            resolver,
            options,
            cwd,
        } = params;

        let commands: Vec<_> = tree.commands().collect_vec();
        let ctx = SymbolContext {
            tree: &tree,
            commands: &commands,
            uri,
            resolver,
            options,
            cwd,
        };

        let environments = Environment::parse(ctx);
        let includes = Include::parse(ctx);
        let components = includes
            .iter()
            .flat_map(|include| include.components(&tree))
            .collect();

        Self {
            tree,
            commands,
            environments,
            includes,
            components,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolContext<'a> {
    tree: &'a Tree,
    commands: &'a [NodeIndex],
    uri: &'a Uri,
    resolver: &'a Resolver,
    options: &'a Options,
    cwd: &'a Path,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnvironmentDelimiter {
    pub node: NodeIndex,
}

impl EnvironmentDelimiter {
    pub fn name<'a>(self, tree: &'a Tree) -> Option<&'a Token> {
        tree.extract_word(self.node, GroupKind::Group, 0)
    }

    pub fn is_math(self, tree: &Tree) -> bool {
        self.is_special(tree, LANGUAGE_DATA.math_environments.iter())
    }

    pub fn is_enum(self, tree: &Tree) -> bool {
        self.is_special(tree, LANGUAGE_DATA.enum_environments.iter())
    }

    fn is_special<'a, I: Iterator<Item = &'a String>>(&self, tree: &Tree, mut values: I) -> bool {
        match self.name(tree) {
            Some(name) => values.any(|env| env == name.text()),
            None => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Environment {
    pub left: EnvironmentDelimiter,
    pub right: EnvironmentDelimiter,
}

impl Environment {
    pub fn is_root(self, tree: &Tree) -> bool {
        self.left
            .name(tree)
            .iter()
            .chain(self.right.name(tree).iter())
            .any(|name| name.text() == "document")
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let mut stack = Vec::new();
        let mut envs = Vec::new();
        for cmd in ctx.commands {
            if let Some((delim, delim_cmd)) = Self::parse_delimiter(ctx.tree, *cmd) {
                if delim_cmd.name.text() == "\\begin" {
                    stack.push(delim);
                } else if let Some(left) = stack.pop() {
                    envs.push(Self { left, right: delim });
                }
            }
        }
        envs
    }

    fn parse_delimiter(tree: &Tree, node: NodeIndex) -> Option<(EnvironmentDelimiter, &Command)> {
        let cmd = tree.as_command(node)?;
        if cmd.name.text() != "\\begin" && cmd.name.text() != "\\end" {
            return None;
        }

        let group = tree.extract_group(node, GroupKind::Group, 0)?;
        if tree.extract_word(node, GroupKind::Group, 0).is_some()
            || tree.children(group).next().is_none()
            || tree.as_group(group)?.right.is_none()
        {
            Some((EnvironmentDelimiter { node }, cmd))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    pub node: NodeIndex,
    pub arg_index: usize,
    pub kind: LatexIncludeKind,
    pub all_targets: Vec<Vec<Uri>>,
    pub include_extension: bool,
}

impl Include {
    pub fn paths<'a>(&self, tree: &'a Tree) -> Vec<&'a Token> {
        tree.extract_comma_separated_words(self.node, GroupKind::Group, self.arg_index)
            .unwrap()
    }

    pub fn components<'a>(&self, tree: &'a Tree) -> impl Iterator<Item = String> + 'a {
        let kind = self.kind;
        self.paths(tree)
            .into_iter()
            .filter_map(move |path| match kind {
                LatexIncludeKind::Package => Some(format!("{}.sty", path.text())),
                LatexIncludeKind::Class => Some(format!("{}.cls", path.text())),
                LatexIncludeKind::Latex
                | LatexIncludeKind::Bibliography
                | LatexIncludeKind::Image
                | LatexIncludeKind::Svg
                | LatexIncludeKind::Pdf
                | LatexIncludeKind::Everything => None,
            })
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let mut includes = Vec::new();
        for cmd in ctx.commands {
            for desc in &LANGUAGE_DATA.include_commands {
                if let Some(include) = Self::parse_single(ctx, *cmd, desc) {
                    includes.push(include);
                }
            }
        }
        includes
    }

    fn parse_single(
        ctx: SymbolContext,
        node: NodeIndex,
        desc: &LatexIncludeCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(node)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        let mut all_targets = Vec::new();
        let paths = ctx
            .tree
            .extract_comma_separated_words(node, GroupKind::Group, desc.index)?;
        for path in paths {
            let mut targets = Vec::new();
            let base_path = Self::base_path(ctx)?;
            let mut relative_path = RelativePath::new(path.text()).to_relative_path_buf();

            let full_path = relative_path.to_path(&base_path);
            targets.push(Uri::from_file_path(full_path).ok()?);

            if let Some(extensions) = desc.kind.extensions() {
                for extension in extensions {
                    let file_name = format!("{}.{}", relative_path.file_stem()?, extension);
                    relative_path.set_file_name(file_name);
                    let full_path = relative_path.to_path(&base_path);
                    targets.push(Uri::from_file_path(full_path).ok()?);
                }
            }

            if let Some(target) = Self::resolve_distro_file(ctx, desc, path.text()) {
                targets.push(target);
            }
            all_targets.push(targets);
        }

        let include = Self {
            node,
            arg_index: desc.index,
            kind: desc.kind,
            all_targets,
            include_extension: desc.include_extension,
        };
        Some(include)
    }

    fn base_path(ctx: SymbolContext) -> Option<PathBuf> {
        let path = if let Some(root_directory) = ctx
            .options
            .latex
            .as_ref()
            .and_then(|opts| opts.root_directory.as_ref())
        {
            root_directory.to_path(ctx.cwd)
        } else {
            let mut path = ctx.uri.to_file_path().ok()?;
            path.pop();
            path
        };
        Some(path)
    }

    fn resolve_distro_file(
        ctx: SymbolContext,
        desc: &LatexIncludeCommand,
        name: &str,
    ) -> Option<Uri> {
        let mut path = ctx.resolver.files_by_name.get(name);
        if let Some(extensions) = desc.kind.extensions() {
            for extension in extensions {
                path = path.or_else(|| {
                    let full_name = format!("{}.{}", name, extension);
                    ctx.resolver.files_by_name.get(&full_name)
                });
            }
        }
        path.and_then(|p| Uri::from_file_path(p).ok())
    }
}
