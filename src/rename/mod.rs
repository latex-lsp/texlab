mod bibtex_entry;
mod latex_command;
mod latex_environment;
mod latex_label;

use self::bibtex_entry::BibtexEntryRenameProvider;
use self::latex_command::LatexCommandRenameProvider;
use self::latex_environment::LatexEnvironmentRenameProvider;
use self::latex_label::LatexLabelRenameProvider;
use crate::feature::{ChoiceProvider, FeatureProvider, FeatureRequest};
use futures::prelude::*;
use futures_boxed::boxed;
use lsp_types::{RenameParams, WorkspaceEdit};

pub struct RenameProvider {
    provider: ChoiceProvider<RenameParams, WorkspaceEdit>,
}

impl RenameProvider {
    pub fn new() -> Self {
        Self {
            provider: ChoiceProvider::new(vec![
                Box::new(BibtexEntryRenameProvider),
                Box::new(LatexCommandRenameProvider),
                Box::new(LatexEnvironmentRenameProvider),
                Box::new(LatexLabelRenameProvider),
            ]),
        }
    }
}

impl FeatureProvider for RenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<RenameParams>,
    ) -> Option<WorkspaceEdit> {
        self.provider.execute(request).await
    }
}
