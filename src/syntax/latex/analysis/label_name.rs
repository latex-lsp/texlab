use latex::LabelReferenceRange;

use crate::syntax::{latex, CstNode};

use super::{LabelName, LatexAnalyzerContext};

pub fn analyze_label_name(
    context: &mut LatexAnalyzerContext,
    node: &latex::SyntaxNode,
) -> Option<()> {
    analyze_label_definition_name(context, node)
        .or_else(|| analyze_label_reference_name(context, node))
        .or_else(|| analyze_label_reference_range_name(context, node))
}

fn analyze_label_definition_name(
    context: &mut LatexAnalyzerContext,
    node: &latex::SyntaxNode,
) -> Option<()> {
    let label = latex::LabelDefinition::cast(node)?;
    let name = label.name()?.word()?;
    context.extras.label_names.push(LabelName {
        text: name.text().into(),
        range: name.text_range(),
        is_definition: true,
    });
    Some(())
}

fn analyze_label_reference_name(
    context: &mut LatexAnalyzerContext,
    node: &latex::SyntaxNode,
) -> Option<()> {
    let label = latex::LabelReference::cast(node)?;
    for name in label.name_list()?.words() {
        context.extras.label_names.push(LabelName {
            text: name.text().into(),
            range: name.text_range(),
            is_definition: false,
        });
    }
    Some(())
}

fn analyze_label_reference_range_name(
    context: &mut LatexAnalyzerContext,
    node: &latex::SyntaxNode,
) -> Option<()> {
    let label = LabelReferenceRange::cast(node)?;
    if let Some(name1) = label.from().and_then(|name| name.word()) {
        context.extras.label_names.push(LabelName {
            text: name1.text().into(),
            range: name1.text_range(),
            is_definition: false,
        });
    }

    if let Some(name2) = label.to().and_then(|name| name.word()) {
        context.extras.label_names.push(LabelName {
            text: name2.text().into(),
            range: name2.text_range(),
            is_definition: false,
        });
    }
    Some(())
}
