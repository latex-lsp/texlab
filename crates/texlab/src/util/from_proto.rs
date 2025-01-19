use std::time::Duration;

use base_db::{
    Config, FeatureParams, Formatter, SymbolEnvironmentConfig, SynctexConfig, Workspace,
};
use completion::CompletionParams;
use definition::DefinitionParams;
use highlights::HighlightParams;
use hover::HoverParams;
use inlay_hints::InlayHintParams;
use references::ReferenceParams;
use rename::RenameParams;
use rowan::TextSize;
use titlecase::titlecase;

use crate::{
    features::completion::ResolveInfo,
    server::options::{BibtexFormatter, CompletionMatcher, LatexFormatter, Options},
};

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

    let hover_markdown = capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.hover.as_ref())
        .and_then(|cap| cap.content_format.as_ref())
        .map_or(false, |cap| {
            // Use the preferred format of the editor instead of just markdown (if available).
            cap.first() == Some(&lsp_types::MarkupKind::Markdown)
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

    let show_document = capabilities
        .window
        .as_ref()
        .and_then(|cap| cap.show_document.as_ref())
        .map_or(false, |cap| cap.support);

    ClientFlags {
        hierarchical_document_symbols,
        completion_markdown,
        hover_markdown,
        completion_snippets,
        completion_kinds,
        completion_always_incomplete,
        configuration_pull,
        configuration_push,
        folding_custom_kinds,
        progress,
        show_document,
    }
}

pub fn rename_params(
    workspace: &Workspace,
    params: lsp_types::TextDocumentPositionParams,
) -> Option<RenameParams> {
    let (feature, offset) =
        feature_params_offset(workspace, params.text_document, params.position)?;

    Some(RenameParams { feature, offset })
}

pub fn hover_params(workspace: &Workspace, params: lsp_types::HoverParams) -> Option<HoverParams> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position_params.text_document,
        params.text_document_position_params.position,
    )?;

    Some(HoverParams { feature, offset })
}

pub fn inlay_hint_params(
    workspace: &Workspace,
    params: lsp_types::InlayHintParams,
) -> Option<InlayHintParams> {
    let feature = feature_params(workspace, params.text_document)?;
    let range = feature.document.line_index.offset_lsp_range(params.range)?;
    Some(InlayHintParams { feature, range })
}

pub fn highlight_params(
    workspace: &Workspace,
    params: lsp_types::DocumentHighlightParams,
) -> Option<HighlightParams<'_>> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position_params.text_document,
        params.text_document_position_params.position,
    )?;

    Some(HighlightParams { feature, offset })
}

pub fn definition_params(
    workspace: &Workspace,
    params: lsp_types::GotoDefinitionParams,
) -> Option<DefinitionParams> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position_params.text_document,
        params.text_document_position_params.position,
    )?;

    Some(DefinitionParams { feature, offset })
}

pub fn completion_params(
    workspace: &Workspace,
    params: lsp_types::CompletionParams,
) -> Option<CompletionParams> {
    let (feature, offset) = feature_params_offset(
        workspace,
        params.text_document_position.text_document,
        params.text_document_position.position,
    )?;

    Some(CompletionParams { feature, offset })
}

pub fn reference_params(
    workspace: &Workspace,
    params: lsp_types::ReferenceParams,
) -> Option<ReferenceParams> {
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

pub fn feature_params(
    workspace: &Workspace,
    text_document: lsp_types::TextDocumentIdentifier,
) -> Option<FeatureParams> {
    let document = workspace.lookup(&text_document.uri)?;
    Some(FeatureParams::new(workspace, document))
}

pub fn feature_params_offset(
    workspace: &Workspace,
    text_document: lsp_types::TextDocumentIdentifier,
    position: lsp_types::Position,
) -> Option<(FeatureParams, TextSize)> {
    let feature = feature_params(workspace, text_document)?;
    let offset = feature.document.line_index.offset_lsp(position)?;
    Some((feature, offset))
}

pub fn completion_resolve_info(item: &mut lsp_types::CompletionItem) -> Option<ResolveInfo> {
    item.data
        .take()
        .and_then(|data| serde_json::from_value(data).ok())
}

pub fn config(value: Options) -> Config {
    let mut config = Config::default();

    config.build.program = value.build.executable.unwrap_or(config.build.program);
    config.build.args = value.build.args.unwrap_or(config.build.args);
    config.build.on_save = value.build.on_save;
    config.build.forward_search_after = value.build.forward_search_after;

    config.build.aux_dir = value
        .build
        .aux_directory
        .or_else(|| value.aux_directory.clone())
        .unwrap_or_else(|| String::from("."));

    config.build.pdf_dir = value
        .build
        .pdf_directory
        .or(value.aux_directory)
        .unwrap_or_else(|| String::from("."));

    config.build.log_dir = value
        .build
        .log_directory
        .unwrap_or_else(|| config.build.pdf_dir.clone());

    config.build.output_filename = value.build.filename;

    config.diagnostics.allowed_patterns = value
        .diagnostics
        .allowed_patterns
        .into_iter()
        .map(|pattern| pattern.0)
        .collect();

    config.diagnostics.ignored_patterns = value
        .diagnostics
        .ignored_patterns
        .into_iter()
        .map(|pattern| pattern.0)
        .collect();

    config.diagnostics.delay = value
        .diagnostics_delay
        .map_or(config.diagnostics.delay, Duration::from_millis);

    config.diagnostics.chktex.on_open = value.chktex.on_open_and_save;
    config.diagnostics.chktex.on_save = value.chktex.on_open_and_save;
    config.diagnostics.chktex.on_edit = value.chktex.on_edit;
    config.diagnostics.chktex.additional_args = value.chktex.additional_args.unwrap_or_default();

    config.formatting.tex_formatter = match value.latex_formatter {
        LatexFormatter::None => Formatter::Null,
        LatexFormatter::Texlab => Formatter::Server,
        LatexFormatter::Latexindent => Formatter::LatexIndent,
        LatexFormatter::TexFmt => Formatter::TexFmt,
    };

    config.formatting.bib_formatter = match value.bibtex_formatter {
        BibtexFormatter::None => Formatter::Null,
        BibtexFormatter::Texlab => Formatter::Server,
        BibtexFormatter::Latexindent => Formatter::LatexIndent,
        BibtexFormatter::TexFmt => Formatter::TexFmt,
    };

    config.formatting.line_length =
        value
            .formatter_line_length
            .map_or(80, |len| if len < 0 { usize::MAX } else { len as usize });

    config.formatting.latex_indent.local = value.latexindent.local;
    config.formatting.latex_indent.modify_line_breaks = value.latexindent.modify_line_breaks;
    config.formatting.latex_indent.replacement = value.latexindent.replacement;

    config.synctex = value
        .forward_search
        .executable
        .zip(value.forward_search.args)
        .map(|(program, args)| SynctexConfig { program, args });

    config.symbols.allowed_patterns = value
        .symbols
        .allowed_patterns
        .into_iter()
        .map(|pattern| pattern.0)
        .collect();

    config.symbols.ignored_patterns = value
        .symbols
        .ignored_patterns
        .into_iter()
        .map(|pattern| pattern.0)
        .collect();

    config.symbols.custom_environments = value
        .symbols
        .custom_environments
        .into_iter()
        .map(|env| {
            let display_name = env.display_name.unwrap_or_else(|| titlecase(&env.name));
            let label = env.label.unwrap_or_default();

            let config = SymbolEnvironmentConfig {
                display_name,
                label,
            };

            (env.name, config)
        })
        .collect();

    config.inlay_hints.label_definitions = value.inlay_hints.label_definitions.unwrap_or(true);
    config.inlay_hints.label_references = value.inlay_hints.label_references.unwrap_or(true);
    config.inlay_hints.max_length = value.inlay_hints.max_length;

    config.completion.matcher = match value.completion.matcher {
        CompletionMatcher::Fuzzy => base_db::MatchingAlgo::Skim,
        CompletionMatcher::FuzzyIgnoreCase => base_db::MatchingAlgo::SkimIgnoreCase,
        CompletionMatcher::Prefix => base_db::MatchingAlgo::Prefix,
        CompletionMatcher::PrefixIgnoreCase => base_db::MatchingAlgo::PrefixIgnoreCase,
    };

    config.syntax.use_file_list = value.build.use_file_list;

    config
        .syntax
        .math_environments
        .extend(value.experimental.math_environments);

    config
        .syntax
        .enum_environments
        .extend(value.experimental.enum_environments);

    config
        .syntax
        .verbatim_environments
        .extend(value.experimental.verbatim_environments);

    config
        .syntax
        .citation_commands
        .extend(value.experimental.citation_commands);

    config
        .syntax
        .label_definition_commands
        .extend(value.experimental.label_definition_commands);

    config
        .syntax
        .label_definition_prefixes
        .extend(value.experimental.label_definition_prefixes);

    config
        .syntax
        .label_reference_commands
        .extend(value.experimental.label_reference_commands);

    config
        .syntax
        .label_reference_prefixes
        .extend(value.experimental.label_reference_prefixes);

    config
        .syntax
        .label_reference_range_commands
        .extend(value.experimental.label_reference_range_commands);

    config
}
