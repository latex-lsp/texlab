use rowan::ast::AstNode;
use rustc_hash::FxHashMap;
use syntax::latex;

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

    fn process_label_number(&mut self, label_number: &latex::LabelNumber) {
        let Some(name) = label_number
            .name()
            .and_then(|group| group.key())
            .map(|key| key.to_string()) else { return };

        let Some(text) = label_number
            .text()
            .map(|node| node.syntax().descendants())
            .into_iter()
            .flatten()
            .find(|node| node.kind() == latex::TEXT || node.kind() == latex::MIXED_GROUP)
            .map(|node| node.text().to_string()) else { return };

        self.label_numbers.insert(name, text);
    }
}
