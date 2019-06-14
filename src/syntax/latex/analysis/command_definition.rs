use crate::syntax::latex::{LatexCommand, LatexContent};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCommandDefinition {
    pub command: Arc<LatexCommand>,
    pub name: Arc<LatexCommand>,
}

impl LatexCommandDefinition {
    pub fn new(command: Arc<LatexCommand>, name: Arc<LatexCommand>) -> Self {
        Self { command, name }
    }

    pub fn parse_all(commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut definitions = Vec::new();
        for command in commands {
            if !DEFINITION_COMMANDS.contains(&command.name.text()) {
                continue;
            }

            if command.args.len() < 2 {
                continue;
            }

            let name = command.args[0].children.iter().next();
            if let Some(LatexContent::Command(name)) = name {
                let definition = Self::new(Arc::clone(&command), Arc::clone(&name));
                definitions.push(definition);
            }
        }
        definitions
    }
}

pub static DEFINITION_COMMANDS: &[&str] =
    &["\\newcommand", "\\renewcommand", "\\DeclareRobustCommand"];
