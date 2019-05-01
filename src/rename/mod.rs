mod bibtex_entry;
mod latex_command;
mod latex_environment;
mod latex_label;

use crate::choice_feature;
use crate::feature::FeatureRequest;
use crate::rename::bibtex_entry::BibtexEntryRenameProvider;
use crate::rename::latex_command::LatexCommandRenameProvider;
use crate::rename::latex_environment::LatexEnvironmentRenameProvider;
use crate::rename::latex_label::LatexLabelRenameProvider;
use lsp_types::{RenameParams, WorkspaceEdit};

pub struct RenameProvider;

impl RenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        choice_feature!(
            &request,
            BibtexEntryRenameProvider,
            LatexCommandRenameProvider,
            LatexEnvironmentRenameProvider,
            LatexLabelRenameProvider
        )
    }
}
