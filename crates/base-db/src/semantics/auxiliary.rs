use rowan::ast::AstNode;
use rustc_hash::FxHashMap;
use syntax::latex::{self, HasCurly};

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub label_numbers: FxHashMap<String, String>,
}

impl Semantics {
    pub fn process_root(&mut self, root: &latex::SyntaxNode) {
        for node in root.descendants() {
            self.process_node(&node);
        }
    }

    fn process_node(&mut self, node: &latex::SyntaxNode) {
        if let Some(label_number) = latex::LabelNumber::cast(node.clone()) {
            self.process_label_number(&label_number);
        }
    }

    fn process_label_number(&mut self, label_number: &latex::LabelNumber) -> Option<()> {
        let name = label_number
            .name()
            .and_then(|group| group.key())
            .map(|key| key.to_string())?;

        let group = label_number.text()?;
        let group = group
            .syntax()
            .children()
            .filter_map(latex::CurlyGroup::cast)
            .find_map(|group| {
                latex::Text::cast(group.syntax().first_child()?)?;
                Some(group)
            })?;

        let text = group.content_text()?.replace(['{', '}'], "");
        self.label_numbers.insert(name, text);
        Some(())
    }
}
