use crate::{
    protocol::{Options, Position, Range, RangeExt, Uri},
    syntax::{lang_data::*, latex::ast::*, text::SyntaxNode},
    tex::Resolver,
};
use itertools::{iproduct, Itertools};
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::Path};

#[derive(Debug, Clone)]
pub struct SymbolTableParams<'a> {
    pub tree: Tree,
    pub uri: &'a Uri,
    pub resolver: &'a Resolver,
    pub options: &'a Options,
    pub current_dir: &'a Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    pub tree: Tree,
    pub commands: Vec<NodeIndex>,
    pub environments: Vec<Environment>,
    pub is_standalone: bool,
    pub includes: Vec<Include>,
    pub components: Vec<String>,
    pub citations: Vec<Citation>,
    pub command_definitions: Vec<CommandDefinition>,
    pub glossary_entries: Vec<GlossaryEntry>,
    pub equations: Vec<Equation>,
    pub inlines: Vec<Inline>,
    pub math_operators: Vec<MathOperator>,
    pub theorem_definitions: Vec<TheoremDefinition>,
    pub sections: Vec<Section>,
    pub labels: Vec<Label>,
    pub label_numberings: Vec<LabelNumbering>,
    pub captions: Vec<Caption>,
    pub items: Vec<Item>,
}

impl SymbolTable {
    pub fn analyze(params: SymbolTableParams) -> Self {
        let SymbolTableParams {
            tree,
            uri,
            resolver,
            options,
            current_dir,
        } = params;

        let commands: Vec<_> = tree.commands().collect();
        let ctx = SymbolContext {
            tree: &tree,
            commands: &commands,
            uri,
            resolver,
            options,
            current_dir,
        };

        let mut environments = None;
        let mut includes = None;
        let mut citations = None;
        let mut command_definitions = None;
        let mut glossary_entries = None;
        let mut equations = None;
        let mut inlines = None;
        let mut math_operators = None;
        let mut theorem_definitions = None;
        let mut sections = None;
        let mut labels = None;
        let mut label_numberings = None;
        let mut captions = None;
        let mut items = None;

        rayon::scope(|s| {
            s.spawn(|_| environments = Some(Environment::parse(ctx)));
            s.spawn(|_| includes = Some(Include::parse(ctx)));
            s.spawn(|_| citations = Some(Citation::parse(ctx)));
            s.spawn(|_| command_definitions = Some(CommandDefinition::parse(ctx)));
            s.spawn(|_| glossary_entries = Some(GlossaryEntry::parse(ctx)));
            s.spawn(|_| equations = Some(Equation::parse(ctx)));
            s.spawn(|_| inlines = Some(Inline::parse(ctx)));
            s.spawn(|_| math_operators = Some(MathOperator::parse(ctx)));
            s.spawn(|_| theorem_definitions = Some(TheoremDefinition::parse(ctx)));
            s.spawn(|_| sections = Some(Section::parse(ctx)));
            s.spawn(|_| labels = Some(Label::parse(ctx)));
            s.spawn(|_| label_numberings = Some(LabelNumbering::parse(ctx)));
            s.spawn(|_| captions = Some(Caption::parse(ctx)));
            s.spawn(|_| items = Some(Item::parse(ctx)));
        });

        let is_standalone = environments
            .as_ref()
            .unwrap()
            .iter()
            .any(|env| env.is_root(&tree));

        let components = includes
            .as_ref()
            .unwrap()
            .iter()
            .flat_map(|include| include.components(&tree))
            .collect();

        Self {
            tree,
            commands,
            environments: environments.unwrap(),
            is_standalone,
            includes: includes.unwrap(),
            components,
            citations: citations.unwrap(),
            command_definitions: command_definitions.unwrap(),
            glossary_entries: glossary_entries.unwrap(),
            equations: equations.unwrap(),
            inlines: inlines.unwrap(),
            math_operators: math_operators.unwrap(),
            theorem_definitions: theorem_definitions.unwrap(),
            sections: sections.unwrap(),
            labels: labels.unwrap(),
            label_numberings: label_numberings.unwrap(),
            captions: captions.unwrap(),
            items: items.unwrap(),
        }
    }

    pub fn is_direct_child(&self, env: &Environment, pos: Position) -> bool {
        env.range(&self.tree).contains(pos)
            && !self
                .environments
                .iter()
                .filter(|e| e.left.parent != env.left.parent)
                .filter(|e| env.range(&self.tree).contains(e.range(&self.tree).start))
                .any(|e| e.range(&self.tree).contains(pos))
    }

    pub fn is_enum_item(&self, enumeration: &Environment, item: Item) -> bool {
        let item_range = self.tree.range(item.parent);
        enumeration.range(&self.tree).contains(item_range.start)
            && !self
                .environments
                .iter()
                .filter(|env| env.left.parent != enumeration.left.parent)
                .filter(|env| env.left.is_enum(&self.tree))
                .filter(|env| {
                    enumeration
                        .range(&self.tree)
                        .contains(env.range(&self.tree).start)
                })
                .any(|env| env.range(&self.tree).contains(item_range.start))
    }

    pub fn find_label_by_range(&self, range: Range) -> Option<&Label> {
        self.labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .filter(|label| label.names(&self.tree).len() == 1)
            .find(|label| range.contains(self.tree.range(label.parent).start))
    }

    pub fn find_label_by_environment(&self, env: &Environment) -> Option<&Label> {
        self.labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .filter(|label| label.names(&self.tree).len() == 1)
            .find(|label| self.is_direct_child(env, self.tree.range(label.parent).start))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolContext<'a> {
    tree: &'a Tree,
    commands: &'a [NodeIndex],
    uri: &'a Uri,
    resolver: &'a Resolver,
    options: &'a Options,
    current_dir: &'a Path,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnvironmentDelimiter {
    pub parent: NodeIndex,
}

impl EnvironmentDelimiter {
    pub fn name(self, tree: &Tree) -> Option<&Token> {
        tree.extract_word(self.parent, GroupKind::Group, 0)
    }

    pub fn is_math(self, tree: &Tree) -> bool {
        self.is_special(tree, LANGUAGE_DATA.math_environments.iter())
    }

    pub fn is_enum(self, tree: &Tree) -> bool {
        self.is_special(tree, LANGUAGE_DATA.enum_environments.iter())
    }

    fn is_special<'a, I: Iterator<Item = &'a String>>(self, tree: &Tree, mut values: I) -> bool {
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

    pub fn range(self, tree: &Tree) -> Range {
        let start = tree.graph[self.left.parent].start();
        let end = tree.graph[self.right.parent].end();
        Range::new(start, end)
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let mut stack = Vec::new();
        let mut envs = Vec::new();
        for parent in ctx.commands {
            if let Some((delim, delim_cmd)) = Self::parse_delimiter(ctx.tree, *parent) {
                if delim_cmd.name.text() == "\\begin" {
                    stack.push(delim);
                } else if let Some(left) = stack.pop() {
                    envs.push(Self { left, right: delim });
                }
            }
        }
        envs
    }

    fn parse_delimiter(tree: &Tree, parent: NodeIndex) -> Option<(EnvironmentDelimiter, &Command)> {
        let cmd = tree.as_command(parent)?;
        if cmd.name.text() != "\\begin" && cmd.name.text() != "\\end" {
            return None;
        }

        let group = tree.extract_group(parent, GroupKind::Group, 0)?;
        if tree.extract_word(parent, GroupKind::Group, 0).is_some()
            || tree.children(group).next().is_none()
            || tree.as_group(group)?.right.is_none()
        {
            Some((EnvironmentDelimiter { parent }, cmd))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    pub parent: NodeIndex,
    pub arg_index: usize,
    pub kind: LatexIncludeKind,
    pub all_targets: Vec<Vec<Uri>>,
    pub include_extension: bool,
}

impl Include {
    pub fn paths<'a>(&self, tree: &'a Tree) -> Vec<&'a Token> {
        tree.extract_comma_separated_words(self.parent, GroupKind::Group, self.arg_index)
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
        iproduct!(ctx.commands, LANGUAGE_DATA.include_commands.iter())
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexIncludeCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        let mut all_targets = Vec::new();
        let paths = ctx
            .tree
            .extract_comma_separated_words(parent, GroupKind::Group, desc.index)?;
        for path in paths {
            let mut targets = Vec::new();
            let base_url = Self::base_url(ctx)?;
            targets.push(base_url.join(path.text()).ok()?.into());

            if let Some(extensions) = desc.kind.extensions() {
                for extension in extensions {
                    let path = format!("{}.{}", path.text(), extension);
                    targets.push(base_url.join(&path).ok()?.into());
                }
            }

            if let Some(target) = Self::resolve_distro_file(ctx, desc, path.text()) {
                targets.push(target);
            }
            all_targets.push(targets);
        }

        let include = Self {
            parent,
            arg_index: desc.index,
            kind: desc.kind,
            all_targets,
            include_extension: desc.include_extension,
        };
        Some(include)
    }

    fn base_url(ctx: SymbolContext) -> Option<Uri> {
        if let Some(root_directory) = ctx
            .options
            .latex
            .as_ref()
            .and_then(|opts| opts.root_directory.as_ref())
        {
            let file_name = ctx.uri.path_segments()?.last()?;
            let path = ctx.current_dir.join(root_directory).join(file_name);
            Uri::from_file_path(path).ok()
        } else {
            Some(ctx.uri.clone())
        }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Citation {
    parent: NodeIndex,
    arg_index: usize,
}

impl Citation {
    pub fn keys(self, tree: &Tree) -> Vec<&Token> {
        tree.extract_comma_separated_words(self.parent, GroupKind::Group, self.arg_index)
            .unwrap()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        iproduct!(ctx.commands, LANGUAGE_DATA.citation_commands.iter())
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexCitationCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        ctx.tree
            .extract_comma_separated_words(parent, GroupKind::Group, desc.index)?;

        Some(Self {
            parent,
            arg_index: desc.index,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub parent: NodeIndex,
    pub definition: NodeIndex,
    pub definition_index: usize,
    pub implementation: NodeIndex,
    pub implementation_index: usize,
    pub arg_count_index: usize,
}

impl CommandDefinition {
    pub fn definition_name(self, tree: &Tree) -> &str {
        tree.as_command(self.definition).unwrap().name.text()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let def = LANGUAGE_DATA.command_definition_commands.iter();
        iproduct!(ctx.commands, def)
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexCommandDefinitionCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        let group_kind = GroupKind::Group;
        let implementation =
            ctx.tree
                .extract_group(parent, group_kind, desc.implementation_index)?;

        let def_group = ctx
            .tree
            .extract_group(parent, group_kind, desc.definition_index)?;

        let mut def_children = ctx.tree.children(def_group);
        let definition = def_children.next()?;
        ctx.tree.as_command(definition)?;
        Some(Self {
            parent,
            definition,
            definition_index: desc.definition_index,
            implementation,
            implementation_index: desc.implementation_index,
            arg_count_index: desc.arg_count_index,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub parent: NodeIndex,
    pub label_index: usize,
    pub kind: LatexGlossaryEntryKind,
}

impl GlossaryEntry {
    pub fn label(self, tree: &Tree) -> &Token {
        tree.extract_word(self.parent, GroupKind::Group, self.label_index)
            .unwrap()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let entry = LANGUAGE_DATA.glossary_entry_definition_commands.iter();
        iproduct!(ctx.commands, entry)
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexGlossaryEntryDefinitionCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        ctx.tree
            .extract_word(parent, GroupKind::Group, desc.label_index)?;

        Some(Self {
            parent,
            label_index: desc.label_index,
            kind: desc.kind,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Equation {
    pub left: NodeIndex,
    pub right: NodeIndex,
}

impl Equation {
    pub fn range(self, tree: &Tree) -> Range {
        let start = tree.range(self.left).start;
        let end = tree.range(self.right).end;
        Range::new(start, end)
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let mut equations = Vec::new();
        let mut left = None;
        for node in ctx.commands {
            let cmd = ctx.tree.as_command(*node).unwrap();
            let name = cmd.name.text();
            if name == "\\[" || name == "\\(" {
                left = Some(node);
            } else if name == "\\]" || name == "\\)" {
                if let Some(begin) = left {
                    equations.push(Self {
                        left: *begin,
                        right: *node,
                    });
                    left = None;
                }
            }
        }
        equations
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Inline {
    pub left: NodeIndex,
    pub right: NodeIndex,
}

impl Inline {
    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let mut inlines = Vec::new();
        let mut left = None;
        for node in ctx
            .tree
            .graph
            .node_indices()
            .filter(|node| ctx.tree.as_math(*node).is_some())
            .sorted_by_key(|node| ctx.tree.graph.node_weight(*node).unwrap().start())
        {
            if let Some(l) = left {
                inlines.push(Inline {
                    left: l,
                    right: node,
                });
                left = None;
            } else {
                left = Some(node);
            }
        }
        inlines
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MathOperator {
    pub parent: NodeIndex,
    pub definition: NodeIndex,
    pub definition_index: usize,
    pub implementation: NodeIndex,
    pub implementation_index: usize,
}

impl MathOperator {
    pub fn definition_name(self, tree: &Tree) -> &str {
        tree.as_command(self.definition).unwrap().name.text()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        iproduct!(ctx.commands, LANGUAGE_DATA.math_operator_commands.iter())
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexMathOperatorCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        let group_kind = GroupKind::Group;
        let def_group = ctx
            .tree
            .extract_group(parent, group_kind, desc.definition_index)?;
        let implementation =
            ctx.tree
                .extract_group(parent, group_kind, desc.implementation_index)?;

        let mut def_children = ctx.tree.children(def_group);
        let definition = def_children.next()?;
        Some(Self {
            parent,
            definition,
            definition_index: desc.definition_index,
            implementation,
            implementation_index: desc.implementation_index,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TheoremDefinition {
    pub parent: NodeIndex,
    pub arg_index: usize,
}

impl TheoremDefinition {
    pub fn name(self, tree: &Tree) -> &Token {
        tree.extract_word(self.parent, GroupKind::Group, self.arg_index)
            .unwrap()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        let thm = LANGUAGE_DATA.theorem_definition_commands.iter();
        iproduct!(ctx.commands, thm)
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexTheoremDefinitionCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        let group_kind = GroupKind::Group;
        ctx.tree.extract_word(parent, group_kind, desc.index)?;

        Some(Self {
            parent,
            arg_index: desc.index,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub parent: NodeIndex,
    pub arg_index: usize,
    pub level: i32,
    pub prefix: Cow<'static, str>,
}

impl Section {
    pub fn print(&self, tree: &Tree) -> Option<String> {
        tree.print_group_content(self.parent, GroupKind::Group, self.arg_index)
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        iproduct!(ctx.commands, LANGUAGE_DATA.section_commands.iter())
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &'static LatexSectionCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        let group_kind = GroupKind::Group;
        ctx.tree.extract_group(parent, group_kind, desc.index)?;

        Some(Self {
            parent,
            arg_index: desc.index,
            level: desc.level,
            prefix: Cow::from(&desc.prefix),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Label {
    pub parent: NodeIndex,
    pub arg_index: usize,
    pub kind: LatexLabelKind,
}

impl Label {
    pub fn names(self, tree: &Tree) -> Vec<&Token> {
        tree.extract_comma_separated_words(self.parent, GroupKind::Group, self.arg_index)
            .unwrap()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        iproduct!(ctx.commands, LANGUAGE_DATA.label_commands.iter())
            .filter_map(|(parent, desc)| Self::parse_single(ctx, *parent, desc))
            .collect()
    }

    fn parse_single(
        ctx: SymbolContext,
        parent: NodeIndex,
        desc: &LatexLabelCommand,
    ) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != desc.name {
            return None;
        }

        ctx.tree
            .extract_comma_separated_words(parent, GroupKind::Group, desc.index)?;

        Some(Self {
            parent,
            arg_index: desc.index,
            kind: desc.kind,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelNumbering {
    pub parent: NodeIndex,
    pub number: String,
}

impl LabelNumbering {
    pub fn name<'a>(&self, tree: &'a Tree) -> &'a Token {
        tree.extract_word(self.parent, GroupKind::Group, 0).unwrap()
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        ctx.commands
            .iter()
            .filter_map(|parent| Self::parse_single(ctx, *parent))
            .collect()
    }

    fn parse_single(ctx: SymbolContext, parent: NodeIndex) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != "\\newlabel" {
            return None;
        }

        ctx.tree.extract_word(parent, GroupKind::Group, 0)?;

        let arg = ctx.tree.extract_group(parent, GroupKind::Group, 1)?;
        let mut analyzer = FirstText::default();
        analyzer.visit(ctx.tree, arg);
        Some(Self {
            parent,
            number: analyzer.text?,
        })
    }
}

#[derive(Debug, Default)]
struct FirstText {
    text: Option<String>,
}

impl Visitor for FirstText {
    fn visit(&mut self, tree: &Tree, node: NodeIndex) {
        if let Some(text) = tree.as_text(node) {
            self.text = Some(text.words.iter().map(Token::text).join(" "));
        }

        if self.text.is_none() {
            tree.walk(self, node);
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Caption {
    pub parent: NodeIndex,
    pub arg_index: usize,
}

impl Caption {
    pub fn print(self, tree: &Tree) -> Option<String> {
        tree.print_group_content(self.parent, GroupKind::Group, self.arg_index)
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        ctx.commands
            .iter()
            .flat_map(|parent| Self::parse_single(ctx, *parent))
            .collect()
    }

    fn parse_single(ctx: SymbolContext, parent: NodeIndex) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != "\\caption" {
            return None;
        }

        ctx.tree.extract_group(parent, GroupKind::Group, 0)?;
        Some(Self {
            parent,
            arg_index: 0,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Item {
    pub parent: NodeIndex,
}

impl Item {
    pub fn name(self, tree: &Tree) -> Option<String> {
        tree.print_group_content(self.parent, GroupKind::Options, 0)
    }

    fn parse(ctx: SymbolContext) -> Vec<Self> {
        ctx.commands
            .iter()
            .filter_map(|parent| Self::parse_single(ctx, *parent))
            .collect()
    }

    fn parse_single(ctx: SymbolContext, parent: NodeIndex) -> Option<Self> {
        let cmd = ctx.tree.as_command(parent)?;
        if cmd.name.text() != "\\item" {
            return None;
        }

        Some(Self { parent })
    }
}
