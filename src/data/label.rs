use crate::outline::Outline;
use crate::syntax::latex::{LatexGroup, LatexLabel, LatexToken};
use crate::syntax::text::{CharStream, SyntaxNode};
use crate::syntax::SyntaxTree;
use crate::workspace::Document;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LabelContext {
    pub section: Option<String>,
    pub caption: Option<String>,
}

impl LabelContext {
    pub fn find(outline: &Outline, document: &Document, label: &LatexLabel) -> LabelContext {
        let section = Self::find_section(outline, document, label);
        let caption = Self::find_caption(document, label);
        Self { section, caption }
    }

    fn find_section(outline: &Outline, document: &Document, label: &LatexLabel) -> Option<String> {
        let section = outline.find(&document.uri, label.start())?;
        let content = &section.command.args[section.index];
        Some(Self::extract(document, content)?)
    }

    fn find_caption(document: &Document, label: &LatexLabel) -> Option<String> {
        if let SyntaxTree::Latex(tree) = &document.tree {
            let environment = tree
                .environments
                .iter()
                .filter(|env| env.left.name().map(LatexToken::text) != Some("document"))
                .find(|env| env.range().contains(label.start()))?;

            let caption = tree
                .captions
                .iter()
                .find(|cap| environment.range().contains(cap.start()))?;
            let content = &caption.command.args[caption.index];
            Some(Self::extract(document, content)?)
        } else {
            None
        }
    }

    fn extract(document: &Document, content: &LatexGroup) -> Option<String> {
        let right = content.right.as_ref()?;
        let range = Range::new_simple(
            content.left.start().line,
            content.left.start().character + 1,
            right.end().line,
            right.end().character - 1,
        );
        Some(CharStream::extract(&document.text, range))
    }

    pub fn documentation(&self) -> Option<Documentation> {
        let text = match (&self.section, &self.caption) {
            (Some(section), Some(caption)) => format!("*{}*  \n{}", section, caption),
            (Some(section), None) => format!("*{}*", section),
            (None, Some(caption)) => caption.to_owned(),
            (None, None) => return None,
        };

        Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text.into(),
        }))
    }
}
