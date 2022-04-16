#[cfg(feature = "citation")]
mod citation;

#[cfg(not(feature = "citation"))]
mod citation {
    use cancellation::CancellationToken;
    use lsp_types::{Hover, HoverParams};

    use crate::features::cursor::CursorContext;

    pub fn find_citation_hover(
        context: &CursorContext<HoverParams>,
        cancellation_token: &CancellationToken,
    ) -> Option<Hover> {
        None
    }
}

mod component;
mod entry_type;
mod field;
mod label;
mod string_ref;

use cancellation::CancellationToken;
use lsp_types::{Hover, HoverParams};

use crate::features::{cursor::CursorContext, hover::citation::find_citation_hover};

use self::{
    component::find_component_hover, entry_type::find_entry_type_hover, field::find_field_hover,
    label::find_label_hover, string_ref::find_string_reference_hover,
};

use super::FeatureRequest;

pub fn find_hover(
    request: FeatureRequest<HoverParams>,
    cabcellation_token: &CancellationToken,
) -> Option<Hover> {
    let context = CursorContext::new(request);
    log::debug!("[Hover] Cursor: {:?}", context.cursor);
    find_label_hover(&context, cabcellation_token)
        .or_else(|| find_citation_hover(&context, cabcellation_token))
        .or_else(|| find_component_hover(&context, cabcellation_token))
        .or_else(|| find_string_reference_hover(&context, cabcellation_token))
        .or_else(|| find_field_hover(&context, cabcellation_token))
        .or_else(|| find_entry_type_hover(&context, cabcellation_token))
}
