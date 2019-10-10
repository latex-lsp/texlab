use super::ast::*;
use crate::syntax::language::*;
use crate::syntax::text::SyntaxNode;
use lsp_types::Range;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexGlossaryEntry {
    pub command: Arc<LatexCommand>,
    pub label_index: usize,
    pub kind: LatexGlossaryEntryKind,
}

impl SyntaxNode for LatexGlossaryEntry {
    fn range(&self) -> Range {
        self.command.range()
    }
}

impl LatexGlossaryEntry {
    pub fn label(&self) -> &LatexToken {
        self.command.extract_word(self.label_index).unwrap()
    }

    fn parse(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut entries = Vec::new();
        for command in commands {
            for LatexGlossaryEntryDefinitionCommand {
                name,
                label_index,
                kind,
            } in &LANGUAGE_DATA.glossary_entry_definition_commands
            {
                if command.name.text() == name && command.has_word(*label_index) {
                    entries.push(Self {
                        command: Arc::clone(&command),
                        label_index: *label_index,
                        kind: *kind,
                    });
                }
            }
        }
        entries
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexGlossaryInfo {
    pub entries: Vec<LatexGlossaryEntry>,
}

impl LatexGlossaryInfo {
    pub fn parse(commands: &[Arc<LatexCommand>]) -> Self {
        Self {
            entries: LatexGlossaryEntry::parse(commands),
        }
    }
}
