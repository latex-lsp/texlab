#[cfg(feature = "citeproc")]
mod citation;
mod component;
mod entry_type;
mod field;
mod label;
mod string_ref;

use cancellation::CancellationToken;
use cfg_if::cfg_if;
use lsp_types::{Hover, HoverParams};

use self::{
    component::find_component_hover, entry_type::find_entry_type_hover, field::find_field_hover,
    label::find_label_hover, string_ref::find_string_reference_hover,
};

use super::FeatureRequest;

pub fn find_hover(
    request: FeatureRequest<HoverParams>,
    token: &CancellationToken,
) -> Option<Hover> {
    let mut hover = find_label_hover(&request, token);

    cfg_if! {
        if #[cfg(feature = "citeproc")] {
            hover = hover.or_else(|| self::citation::find_citation_hover(&request, token));
        }
    }

    hover = hover
        .or_else(|| find_component_hover(&request, token))
        .or_else(|| find_string_reference_hover(&request, token))
        .or_else(|| find_field_hover(&request, token))
        .or_else(|| find_entry_type_hover(&request, token));

    hover
}
