use lsp_types::ClientCapabilities;

pub trait ClientCapabilitiesExt {
    fn has_definition_link_support(&self) -> bool;

    fn has_hierarchical_document_symbol_support(&self) -> bool;

    fn has_work_done_progress(&self) -> bool;
}

impl ClientCapabilitiesExt for ClientCapabilities {
    fn has_definition_link_support(&self) -> bool {
        self.text_document
            .as_ref()
            .and_then(|cap| cap.definition.as_ref())
            .and_then(|cap| cap.link_support)
            == Some(true)
    }

    fn has_hierarchical_document_symbol_support(&self) -> bool {
        self.text_document
            .as_ref()
            .and_then(|cap| cap.document_symbol.as_ref())
            .and_then(|cap| cap.hierarchical_document_symbol_support)
            == Some(true)
    }

    fn has_work_done_progress(&self) -> bool {
        self.window.as_ref().and_then(|cap| cap.work_done_progress) == Some(true)
    }
}
