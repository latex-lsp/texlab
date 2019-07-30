use crate::outline::Outline;
use lsp_types::*;
use texlab_syntax::*;
use texlab_workspace::Document;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LabelContext {
    pub section: Option<String>,
    pub caption: Option<String>,
}

impl LabelContext {
    pub fn find(outline: &Outline, document: &Document, position: Position) -> LabelContext {
        let section = Self::find_section(outline, document, position);
        let caption = Self::find_caption(document, position);
        Self { section, caption }
    }

    fn find_section(outline: &Outline, document: &Document, position: Position) -> Option<String> {
        let section = outline.find(&document.uri, position)?;
        let content = &section.command.args[section.index];
        Some(Self::extract(document, content)?)
    }

    fn find_caption(document: &Document, position: Position) -> Option<String> {
        if let SyntaxTree::Latex(tree) = &document.tree {
            let environment = tree
                .environments
                .iter()
                .filter(|env| env.left.name().map(LatexToken::text) != Some("document"))
                .find(|env| env.range().contains(position))?;

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

    pub fn documentation(&self) -> Option<MarkupContent> {
        let text = match (&self.section, &self.caption) {
            (Some(section), Some(caption)) => format!("*{}*  \n{}", section, caption),
            (Some(section), None) => format!("*{}*", section),
            (None, Some(caption)) => caption.to_owned(),
            (None, None) => return None,
        };

        Some(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text.into(),
        })
    }
}
