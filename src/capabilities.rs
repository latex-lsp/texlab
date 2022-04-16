use lsp_types::{ClientCapabilities, MarkupKind};

pub trait ClientCapabilitiesExt {
    fn has_definition_link_support(&self) -> bool;

    fn has_hierarchical_document_symbol_support(&self) -> bool;

    fn has_work_done_progress_support(&self) -> bool;

    fn has_hover_markdown_support(&self) -> bool;

    fn has_pull_configuration_support(&self) -> bool;

    fn has_push_configuration_support(&self) -> bool;

    fn has_file_watching_support(&self) -> bool;
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

    fn has_work_done_progress_support(&self) -> bool {
        self.window.as_ref().and_then(|cap| cap.work_done_progress) == Some(true)
    }

    fn has_hover_markdown_support(&self) -> bool {
        self.text_document
            .as_ref()
            .and_then(|cap| cap.hover.as_ref())
            .and_then(|cap| cap.content_format.as_ref())
            .filter(|formats| formats.contains(&MarkupKind::Markdown))
            .is_some()
    }

    fn has_pull_configuration_support(&self) -> bool {
        self.workspace.as_ref().and_then(|cap| cap.configuration) == Some(true)
    }

    fn has_push_configuration_support(&self) -> bool {
        self.workspace
            .as_ref()
            .and_then(|cap| cap.did_change_configuration)
            .and_then(|cap| cap.dynamic_registration)
            == Some(true)
    }

    fn has_file_watching_support(&self) -> bool {
        self.workspace
            .as_ref()
            .and_then(|cap| cap.did_change_watched_files)
            .and_then(|cap| cap.dynamic_registration)
            == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        DocumentSymbolClientCapabilities, GotoCapability, HoverClientCapabilities,
        TextDocumentClientCapabilities, WindowClientCapabilities,
    };

    #[test]
    fn test_has_definition_link_support_true() {
        let capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                definition: Some(GotoCapability {
                    link_support: Some(true),
                    ..GotoCapability::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_definition_link_support());
    }

    #[test]
    fn test_has_definition_link_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_definition_link_support());
    }

    #[test]
    fn test_has_hierarchical_document_symbol_support_true() {
        let capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                document_symbol: Some(DocumentSymbolClientCapabilities {
                    hierarchical_document_symbol_support: Some(true),
                    ..DocumentSymbolClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_hierarchical_document_symbol_support());
    }

    #[test]
    fn test_has_hierarchical_document_symbol_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_hierarchical_document_symbol_support());
    }

    #[test]
    fn test_has_work_done_progress_support_true() {
        let capabilities = ClientCapabilities {
            window: Some(WindowClientCapabilities {
                work_done_progress: Some(true),
                ..WindowClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_work_done_progress_support());
    }

    #[test]
    fn test_has_work_done_progress_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_work_done_progress_support());
    }

    #[test]
    fn test_has_hover_markdown_support_true() {
        let capabilities = ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                hover: Some(HoverClientCapabilities {
                    content_format: Some(vec![MarkupKind::PlainText, MarkupKind::Markdown]),
                    ..HoverClientCapabilities::default()
                }),
                ..TextDocumentClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        };
        assert!(capabilities.has_hover_markdown_support());
    }

    #[test]
    fn test_has_hover_markdown_support_false() {
        let capabilities = ClientCapabilities::default();
        assert!(!capabilities.has_hover_markdown_support());
    }
}
