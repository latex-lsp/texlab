use base_db::{FeatureParams, Workspace};
use completion::CompletionParams;
use definition::DefinitionParams;
use highlights::HighlightParams;
use hover::HoverParams;
use inlay_hints::InlayHintParams;
use references::ReferenceParams;
use rename::RenameParams;
use rowan::TextSize;

use crate::features::completion::ResolveInfo;

use super::{line_index_ext::LineIndexExt, ClientFlags};

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

pub fn rename_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::TextDocumentPositionParams,
) -> Option<RenameParams<'a>> {
    let (feature, offset) =
        feature_params_offset(workspace, params.text_document, params.position)?;

    Some(RenameParams { feature, offset })
}

pub fn hover_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::HoverParams,
) -> Option<HoverParams<'a>> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position_params.text_document,
        params.text_document_position_params.position,
    )?;

    Some(HoverParams { feature, offset })
}

pub fn inlay_hint_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::InlayHintParams,
) -> Option<InlayHintParams> {
    let feature = feature_params(workspace, params.text_document)?;
    let range = feature.document.line_index.offset_lsp_range(params.range)?;
    Some(InlayHintParams { feature, range })
}

pub fn highlight_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::DocumentHighlightParams,
) -> Option<HighlightParams<'a>> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position_params.text_document,
        params.text_document_position_params.position,
    )?;

    Some(HighlightParams { feature, offset })
}

pub fn definition_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::GotoDefinitionParams,
) -> Option<DefinitionParams<'a>> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position_params.text_document,
        params.text_document_position_params.position,
    )?;

    Some(DefinitionParams { feature, offset })
}

pub fn completion_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::CompletionParams,
) -> Option<CompletionParams<'a>> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position.text_document,
        params.text_document_position.position,
    )?;

    Some(CompletionParams { feature, offset })
}

pub fn reference_params<'a>(
    workspace: &'a Workspace,
    params: lsp_types::ReferenceParams,
) -> Option<ReferenceParams<'a>> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position.text_document,
        params.text_document_position.position,
    )?;

    let include_declaration = params.context.include_declaration;
    Some(ReferenceParams {
        feature,
        offset,
        include_declaration,
    })
}

pub fn feature_params<'a>(
    workspace: &'a Workspace,
    text_document: lsp_types::TextDocumentIdentifier,
) -> Option<FeatureParams<'a>> {
    let document = workspace.lookup(&text_document.uri)?;
    Some(FeatureParams::new(workspace, document))
}

pub fn feature_params_offset<'a>(
    workspace: &'a Workspace,
    text_document: lsp_types::TextDocumentIdentifier,
    position: lsp_types::Position,
) -> Option<(FeatureParams<'a>, TextSize)> {
    let feature = feature_params(workspace, text_document)?;
    let offset = feature.document.line_index.offset_lsp(position)?;
    Some((feature, offset))
}

pub fn completion_resolve_info(item: &mut lsp_types::CompletionItem) -> Option<ResolveInfo> {
    item.data
        .take()
        .and_then(|data| serde_json::from_value(data).ok())
}
