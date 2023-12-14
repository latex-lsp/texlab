use super::ClientFlags;

pub fn client_flags(
    capabilities: lsp_types::ClientCapabilities,
    info: Option<lsp_types::ClientInfo>,
) -> ClientFlags {
    let hierarchical_document_symbols = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.document_symbol.as_ref())
        .and_then(|cap| cap.hierarchical_document_symbol_support)
        .unwrap_or(false);

    let completion_markdown = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.completion.as_ref())
        .and_then(|cap| cap.completion_item.as_ref())
        .and_then(|cap| cap.documentation_format.as_ref())
        .map_or(false, |formats| {
            formats.contains(&lsp_types::MarkupKind::Markdown)
        });

    let completion_snippets = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.completion.as_ref())
        .and_then(|cap| cap.completion_item.as_ref())
        .and_then(|cap| cap.snippet_support)
        .unwrap_or(false);

    let completion_kinds = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.completion.as_ref())
        .and_then(|cap| cap.completion_item_kind.as_ref())
        .and_then(|cap| cap.value_set.clone())
        .unwrap_or_default();

    let completion_always_incomplete = info.map_or(false, |info| info.name == "Visual Studio Code");

    let hover_markdown = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.hover.as_ref())
        .and_then(|cap| cap.content_format.as_ref())
        .map_or(false, |formats| {
            formats.contains(&lsp_types::MarkupKind::Markdown)
        });

    let configuration_pull = capabilities
        .workspace
        .as_ref()
        .and_then(|cap| cap.configuration)
        .unwrap_or(false);

    let configuration_push = capabilities
        .workspace
        .as_ref()
        .and_then(|cap| cap.did_change_configuration)
        .and_then(|cap| cap.dynamic_registration)
        .unwrap_or(false);

    let definition_link = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.definition)
        .and_then(|cap| cap.link_support)
        .unwrap_or(false);

    let folding_custom_kinds = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.folding_range.as_ref())
        .and_then(|cap| cap.folding_range_kind.as_ref())
        .and_then(|cap| cap.value_set.as_ref())
        .is_some();

    let progress = capabilities
        .window
        .as_ref()
        .and_then(|cap| cap.work_done_progress)
        .unwrap_or(false);

    ClientFlags {
        hierarchical_document_symbols,
        completion_markdown,
        completion_snippets,
        completion_kinds,
        completion_always_incomplete,
        hover_markdown,
        configuration_pull,
        configuration_push,
        definition_link,
        folding_custom_kinds,
        progress,
    }
}
