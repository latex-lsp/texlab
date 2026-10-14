use base_db::DocumentData;
use rowan::{TextRange, ast::AstNode};
use syntax::latex;

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let feature = &context.params.feature;
    let data = feature.document.data.as_tex()?;
    let root = data.root_node();

    // Find the token at the cursor position
    let token = root.token_at_offset(context.params.offset).right_biased()?;

    // Walk up ancestors to find a CurlyGroupWord whose parent is an AcronymReference
    let name_group = token
        .parent_ancestors()
        .find_map(latex::CurlyGroupWord::cast)?;
    latex::AcronymReference::cast(name_group.syntax().parent()?)?;

    let key = name_group.key()?;
    let key_text = key.to_string();
    let origin_selection_range = latex::small_range(&name_group);

    // Search all project documents for acronym definitions
    for document in &feature.project.documents {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let results = data
            .root_node()
            .descendants()
            .filter_map(|node| {
                process_definition(node.clone(), &key_text)
                    .or_else(|| process_declaration(node, &key_text))
            })
            .map(|(target_range, target_selection_range)| DefinitionResult {
                origin_selection_range,
                target: document,
                target_range,
                target_selection_range,
            });

        context.results.extend(results);
    }

    Some(())
}

fn process_definition(node: latex::SyntaxNode, key: &str) -> Option<(TextRange, TextRange)> {
    let def = latex::AcronymDefinition::cast(node)?;
    let name = def.name()?;
    let name_key = name.key()?;
    if name_key.to_string() == key {
        Some((latex::small_range(&def), latex::small_range(&name)))
    } else {
        None
    }
}

fn process_declaration(node: latex::SyntaxNode, key: &str) -> Option<(TextRange, TextRange)> {
    let decl = latex::AcronymDeclaration::cast(node)?;
    let name = decl.name()?;
    let name_key = name.key()?;
    if name_key.to_string() == key {
        Some((latex::small_range(&decl), latex::small_range(&name)))
    } else {
        None
    }
}
