use lsp_types::{CompletionTextEdit, TextEdit};

pub trait CompletionTextEditExt {
    fn text_edit(&self) -> Option<&TextEdit>;
}

impl CompletionTextEditExt for CompletionTextEdit {
    fn text_edit(&self) -> Option<&TextEdit> {
        let CompletionTextEdit::Edit(edit) = self;
        Some(edit)
    }
}
