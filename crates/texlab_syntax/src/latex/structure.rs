use super::ast::*;
use crate::language::*;
use crate::text::{CharStream, SyntaxNode};
use itertools::Itertools;
use std::sync::Arc;
use texlab_protocol::{Range, RangeExt};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSection {
    pub command: Arc<LatexCommand>,
    pub index: usize,
    pub level: i32,
    pub prefix: &'static str,
}

impl LatexSection {
    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut sections = Vec::new();
        for command in commands {
            for LatexSectionCommand {
                name,
                index,
                level,
                prefix,
            } in &LANGUAGE_DATA.section_commands
            {
                if command.name.text() == name && command.args.len() > *index {
                    sections.push(Self {
                        command: Arc::clone(command),
                        index: *index,
                        level: *level,
                        prefix: prefix.as_ref(),
                    })
                }
            }
        }
        sections
    }

    pub fn extract_text(&self, text: &str) -> Option<String> {
        let content = &self.command.args[self.index];
        let right = content.right.as_ref()?;
        let range = Range::new_simple(
            content.left.start().line,
            content.left.start().character + 1,
            right.end().line,
            right.end().character - 1,
        );
        Some(CharStream::extract(&text, range))
    }
}

impl SyntaxNode for LatexSection {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabel {
    pub command: Arc<LatexCommand>,
    index: usize,
    pub kind: LatexLabelKind,
}

impl LatexLabel {
    pub fn names(&self) -> Vec<&LatexToken> {
        self.command.extract_comma_separated_words(self.index)
    }

    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut labels = Vec::new();
        for command in commands {
            for LatexLabelCommand { name, index, kind } in &LANGUAGE_DATA.label_commands {
                if command.name.text() == name && command.has_comma_separated_words(*index) {
                    labels.push(Self {
                        command: Arc::clone(command),
                        index: *index,
                        kind: *kind,
                    });
                }
            }
        }
        labels
    }
}

impl SyntaxNode for LatexLabel {
    fn range(&self) -> Range {
        self.command.range
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelNumbering {
    pub command: Arc<LatexCommand>,
    pub number: String,
}

impl LatexLabelNumbering {
    pub fn name(&self) -> &LatexToken {
        self.command.extract_word(0).unwrap()
    }

    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        commands
            .iter()
            .map(Arc::clone)
            .filter_map(Self::parse_single)
            .collect()
    }

    fn parse_single(command: Arc<LatexCommand>) -> Option<Self> {
        #[derive(Debug, Default)]
        struct FirstText {
            text: Option<Arc<LatexText>>,
        }

        impl LatexVisitor for FirstText {
            fn visit_root(&mut self, root: Arc<LatexRoot>) {
                LatexWalker::walk_root(self, root);
            }

            fn visit_group(&mut self, group: Arc<LatexGroup>) {
                LatexWalker::walk_group(self, group);
            }

            fn visit_command(&mut self, command: Arc<LatexCommand>) {
                LatexWalker::walk_command(self, command);
            }

            fn visit_text(&mut self, text: Arc<LatexText>) {
                if self.text.is_none() {
                    self.text = Some(text);
                }
            }

            fn visit_comma(&mut self, comma: Arc<LatexComma>) {
                LatexWalker::walk_comma(self, comma);
            }

            fn visit_math(&mut self, math: Arc<LatexMath>) {
                LatexWalker::walk_math(self, math);
            }
        }

        if command.name.text() != "\\newlabel" || !command.has_word(0) {
            return None;
        }

        let mut analyzer = FirstText::default();
        analyzer.visit_group(Arc::clone(command.args.get(1)?));
        let number = analyzer
            .text?
            .words
            .iter()
            .map(|word| word.text())
            .join(" ");

        Some(Self {
            command: Arc::clone(&command),
            number,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCaption {
    pub command: Arc<LatexCommand>,
    pub index: usize,
}

impl LatexCaption {
    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut captions = Vec::new();
        for command in commands {
            if command.name.text() == "\\caption" && !command.args.is_empty() {
                captions.push(Self {
                    command: Arc::clone(&command),
                    index: 0,
                });
            }
        }
        captions
    }
}

impl SyntaxNode for LatexCaption {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexItem {
    pub command: Arc<LatexCommand>,
}

impl LatexItem {
    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut items = Vec::new();
        for command in commands {
            if command.name.text() == "\\item" {
                items.push(Self {
                    command: Arc::clone(&command),
                });
            }
        }
        items
    }

    pub fn name(&self) -> Option<String> {
        if let Some(options) = self.command.options.get(0) {
            if options.children.len() == 1 {
                if let LatexContent::Text(text) = &options.children[0] {
                    return Some(text.words.iter().map(|word| word.text()).join(" "));
                }
            }
        }
        None
    }
}

impl SyntaxNode for LatexItem {
    fn range(&self) -> Range {
        self.command.range()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexStructureInfo {
    pub sections: Vec<LatexSection>,
    pub labels: Vec<LatexLabel>,
    pub label_numberings: Vec<LatexLabelNumbering>,
    pub captions: Vec<LatexCaption>,
    pub items: Vec<LatexItem>,
}

impl LatexStructureInfo {
    pub fn parse(commands: &[Arc<LatexCommand>]) -> Self {
        Self {
            sections: LatexSection::parse(commands),
            labels: LatexLabel::parse(commands),
            label_numberings: LatexLabelNumbering::parse(commands),
            captions: LatexCaption::parse(commands),
            items: LatexItem::parse(commands),
        }
    }
}
