mod ast;
mod env;
mod finder;
mod glossary;
mod lexer;
mod math;
mod parser;
mod printer;
mod structure;

pub use self::ast::*;
pub use self::env::*;
pub use self::finder::LatexNode;
pub use self::glossary::*;
pub use self::math::*;
pub use self::printer::LatexPrinter;
pub use self::structure::*;

use self::finder::LatexFinder;
use self::lexer::LatexLexer;
use self::parser::LatexParser;
use super::language::*;
use super::text::SyntaxNode;
use super::SyntaxTreeContext;
use crate::range::RangeExt;
use crate::workspace::Uri;
use lsp_types::{Position, Range};
use path_clean::PathClean;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Default)]
struct LatexCommandAnalyzer {
    commands: Vec<Arc<LatexCommand>>,
}

impl LatexCommandAnalyzer {
    fn parse(root: Arc<LatexRoot>) -> Vec<Arc<LatexCommand>> {
        let mut analyzer = Self::default();
        analyzer.visit_root(root);
        analyzer.commands
    }
}

impl LatexVisitor for LatexCommandAnalyzer {
    fn visit_root(&mut self, root: Arc<LatexRoot>) {
        LatexWalker::walk_root(self, root);
    }

    fn visit_group(&mut self, group: Arc<LatexGroup>) {
        LatexWalker::walk_group(self, group);
    }

    fn visit_command(&mut self, command: Arc<LatexCommand>) {
        self.commands.push(Arc::clone(&command));
        LatexWalker::walk_command(self, command);
    }

    fn visit_text(&mut self, text: Arc<LatexText>) {
        LatexWalker::walk_text(self, text);
    }

    fn visit_comma(&mut self, comma: Arc<LatexComma>) {
        LatexWalker::walk_comma(self, comma);
    }

    fn visit_math(&mut self, math: Arc<LatexMath>) {
        LatexWalker::walk_math(self, math);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitation {
    pub command: Arc<LatexCommand>,
    pub index: usize,
}

impl LatexCitation {
    pub fn keys(&self) -> Vec<&LatexToken> {
        self.command.extract_comma_separated_words(0)
    }

    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut citations = Vec::new();
        for command in commands {
            for LatexCitationCommand { name, index } in &LANGUAGE_DATA.citation_commands {
                if command.name.text() == name && command.has_comma_separated_words(*index) {
                    citations.push(Self {
                        command: Arc::clone(command),
                        index: *index,
                    });
                }
            }
        }
        citations
    }
}

impl SyntaxNode for LatexCitation {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInclude {
    pub command: Arc<LatexCommand>,
    pub index: usize,
    pub kind: LatexIncludeKind,
    pub all_targets: Vec<Vec<Uri>>,
    pub include_extension: bool,
}

impl LatexInclude {
    pub fn paths(&self) -> Vec<&LatexToken> {
        self.command.extract_comma_separated_words(self.index)
    }

    pub fn components(&self) -> Vec<String> {
        let mut components = Vec::new();
        for path in self.paths() {
            match self.kind {
                LatexIncludeKind::Package => components.push(format!("{}.sty", path.text())),
                LatexIncludeKind::Class => components.push(format!("{}.cls", path.text())),
                LatexIncludeKind::Latex
                | LatexIncludeKind::Bibliography
                | LatexIncludeKind::Image
                | LatexIncludeKind::Svg
                | LatexIncludeKind::Pdf
                | LatexIncludeKind::Everything => (),
            }
        }
        components
    }

    fn parse(context: SyntaxTreeContext, commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut includes = Vec::new();
        for command in commands {
            for description in &LANGUAGE_DATA.include_commands {
                if let Some(include) = Self::parse_single(context, &command, &description) {
                    includes.push(include);
                }
            }
        }
        includes
    }

    fn parse_single(
        context: SyntaxTreeContext,
        command: &Arc<LatexCommand>,
        description: &LatexIncludeCommand,
    ) -> Option<Self> {
        if command.name.text() != description.name {
            return None;
        }

        if command.args.len() <= description.index {
            return None;
        }

        let mut all_targets = Vec::new();
        for relative_path in command.extract_comma_separated_words(description.index) {
            let mut path = context.uri.to_file_path().ok()?;
            path.pop();
            path.push(relative_path.text());
            path = PathBuf::from(path.to_string_lossy().into_owned().replace('\\', "/"));
            path = path.clean();
            let path = path.to_str()?.to_owned();

            let mut targets = Vec::new();
            targets.push(Uri::from_file_path(&path).ok()?);
            if let Some(extensions) = description.kind.extensions() {
                for extension in extensions {
                    let path = format!("{}.{}", &path, extension);
                    targets.push(Uri::from_file_path(&path).ok()?);
                }
            }
            all_targets.push(targets);
        }

        for name in command.extract_comma_separated_words(description.index) {
            let mut path = context.resolver.files_by_name.get(&name.span.text);
            if let Some(extensions) = description.kind.extensions() {
                for extension in extensions {
                    path = path.or_else(|| {
                        let full_name = format!("{}.{}", name.text(), extension);
                        context.resolver.files_by_name.get(&full_name)
                    });
                }
            }

            if let Some(path) = path {
                all_targets.push(vec![Uri::from_file_path(&path).ok()?]);
            }
        }

        let include = Self {
            command: Arc::clone(command),
            index: description.index,
            kind: description.kind,
            all_targets,
            include_extension: description.include_extension,
        };
        Some(include)
    }
}

impl SyntaxNode for LatexInclude {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCommandDefinition {
    pub command: Arc<LatexCommand>,
    pub definition: Arc<LatexCommand>,
    pub definition_index: usize,
    pub implementation: Arc<LatexGroup>,
    pub implementation_index: usize,
    pub argument_count_index: usize,
}

impl LatexCommandDefinition {
    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut definitions = Vec::new();
        for command in commands {
            for LatexCommandDefinitionCommand {
                name,
                definition_index,
                argument_count_index,
                implementation_index,
            } in &LANGUAGE_DATA.command_definition_commands
            {
                if command.name.text() == name
                    && command.args.len() > *definition_index
                    && command.args.len() > *implementation_index
                {
                    let definition = command.args[0].children.iter().next();
                    if let Some(LatexContent::Command(definition)) = definition {
                        definitions.push(Self {
                            command: Arc::clone(command),
                            definition: Arc::clone(definition),
                            definition_index: *definition_index,
                            implementation: Arc::clone(&command.args[*implementation_index]),
                            implementation_index: *implementation_index,
                            argument_count_index: *argument_count_index,
                        })
                    }
                }
            }
        }
        definitions
    }
}

impl SyntaxNode for LatexCommandDefinition {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSyntaxTree {
    pub root: Arc<LatexRoot>,
    pub commands: Vec<Arc<LatexCommand>>,
    pub includes: Vec<LatexInclude>,
    pub components: Vec<String>,
    pub env: LatexEnvironmentInfo,
    pub structure: LatexStructureInfo,
    pub citations: Vec<LatexCitation>,
    pub math: LatexMathInfo,
    pub command_definitions: Vec<LatexCommandDefinition>,
    pub glossary: LatexGlossaryInfo,
}

impl LatexSyntaxTree {
    pub fn parse(context: SyntaxTreeContext, text: &str) -> Self {
        let lexer = LatexLexer::new(text);
        let mut parser = LatexParser::new(lexer);
        let root = Arc::new(parser.root());
        let commands = LatexCommandAnalyzer::parse(Arc::clone(&root));
        let includes = LatexInclude::parse(context, &commands);
        let components = includes.iter().flat_map(LatexInclude::components).collect();
        let env = LatexEnvironmentInfo::parse(&commands);
        let structure = LatexStructureInfo::parse(&commands);
        let citations = LatexCitation::parse(&commands);
        let math = LatexMathInfo::parse(Arc::clone(&root), &commands);
        let command_definitions = LatexCommandDefinition::parse(&commands);
        let glossary = LatexGlossaryInfo::parse(&commands);
        Self {
            root,
            commands,
            includes,
            components,
            env,
            structure,
            citations,
            math,
            command_definitions,
            glossary,
        }
    }

    pub fn find(&self, position: Position) -> Vec<LatexNode> {
        let mut finder = LatexFinder::new(position);
        finder.visit_root(Arc::clone(&self.root));
        finder.results
    }

    pub fn find_command_by_name(&self, position: Position) -> Option<Arc<LatexCommand>> {
        for result in self.find(position) {
            if let LatexNode::Command(command) = result {
                if command.name.range().contains(position)
                    && command.name.start().character != position.character
                {
                    return Some(command);
                }
            }
        }
        None
    }

    pub fn find_label_by_range(&self, range: Range) -> Option<&LatexLabel> {
        self.structure
            .labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .filter(|label| label.names().len() == 1)
            .find(|label| range.contains(label.start()))
    }

    pub fn find_label_by_environment(&self, environment: &LatexEnvironment) -> Option<&LatexLabel> {
        self.structure
            .labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .filter(|label| label.names().len() == 1)
            .find(|label| self.is_direct_child(environment, label.start()))
    }

    pub fn is_enumeration_item(&self, enumeration: &LatexEnvironment, item: &LatexItem) -> bool {
        enumeration.range().contains(item.start())
            && !self
                .env
                .environments
                .iter()
                .filter(|env| *env != enumeration)
                .filter(|env| env.left.is_enum() && enumeration.range().contains(env.start()))
                .any(|env| env.range().contains(item.start()))
    }

    pub fn is_direct_child(&self, environment: &LatexEnvironment, position: Position) -> bool {
        environment.range().contains(position)
            && !self
                .env
                .environments
                .iter()
                .filter(|env| *env != environment)
                .filter(|env| environment.range().contains(env.start()))
                .any(|env| env.range().contains(position))
    }
}

pub fn extract_group(content: &LatexGroup) -> String {
    if content.children.is_empty() || content.right.is_none() {
        return String::new();
    }

    let mut printer = LatexPrinter::new(content.children[0].start());
    for child in &content.children {
        child.accept(&mut printer);
    }
    printer.output
}
