use rowan::ast::AstNode;
use rustc_hash::FxHashMap;
use syntax::latex::{self, HasCurly};

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub label_numbers: FxHashMap<String, String>,
    pub section_numbers: FxHashMap<String, String>,
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
        if let Some(toc_line) = latex::TocContentsLine::cast(node.clone()) {
            self.process_toc_line(&toc_line);
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

    fn process_toc_line(&mut self, toc_line: &latex::TocContentsLine) -> Option<()> {
        let unit = toc_line.unit().and_then(|u| u.content_text())?.to_string();

        if ["section", "subsection"].contains(&unit.as_str()) {
            let line = toc_line.line()?;
            let name = line
                .syntax()
                .children()
                .find_map(|child| { latex::Text::cast(child.clone())?; Some(child)})?;
            let name = name.to_string();

            let num_line = line
                .syntax()
                .children()
                .find_map(|child| latex::TocNumberLine::cast(child.clone()))?;
            let number = num_line
                .number()?;
            let number = number.content_text()?.replace(['{', '}'], "");
            self.section_numbers.insert(name, number);
        }
        Some(())
    }
}
