mod bibtex;
mod factory;
mod latex;
mod quality;

use self::bibtex::entry_type::BibtexEntryTypeCompletionProvider;
use self::bibtex::field_name::BibtexFieldNameCompletionProvider;
use self::bibtex::kernel_command::BibtexKernelCommandCompletionProvider;
use self::latex::argument_symbol::LatexArgumentSymbolCompletionProvider;
use self::latex::begin_command::LatexBeginCommandCompletionProvider;
use self::latex::citation::LatexCitationCompletionProvider;
use self::latex::color::LatexColorCompletionProvider;
use self::latex::color_model::LatexColorModelCompletionProvider;
use self::latex::command_symbol::LatexCommandSymbolCompletionProvider;
use self::latex::import::{LatexClassImportProvider, LatexPackageImportProvider};
use self::latex::include::LatexIncludeCompletionProvider;
use self::latex::kernel_command::LatexKernelCommandCompletionProvider;
use self::latex::kernel_environment::LatexKernelEnvironmentCompletionProvider;
use self::latex::label::LatexLabelCompletionProvider;
use self::latex::pgf_library::LatexPgfLibraryCompletionProvider;
use self::latex::tikz_command::LatexTikzCommandCompletionProvider;
use self::latex::tikz_library::LatexTikzLibraryCompletionProvider;
use self::latex::user_command::LatexUserCommandCompletionProvider;
use self::quality::OrderByQualityCompletionProvider;
use crate::concat_feature;
use crate::feature::FeatureRequest;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionParams};

pub const COMPLETION_LIMIT: usize = 50;

pub struct CompletionProvider;

impl CompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        let items = OrderByQualityCompletionProvider::execute(request, async move |_| {
            concat_feature!(
                &request,
                BibtexEntryTypeCompletionProvider,
                BibtexFieldNameCompletionProvider,
                BibtexKernelCommandCompletionProvider,
                LatexKernelEnvironmentCompletionProvider,
                LatexArgumentSymbolCompletionProvider,
                LatexPgfLibraryCompletionProvider,
                LatexTikzLibraryCompletionProvider,
                LatexColorCompletionProvider,
                LatexColorModelCompletionProvider,
                LatexLabelCompletionProvider,
                LatexCitationCompletionProvider,
                LatexIncludeCompletionProvider,
                LatexClassImportProvider,
                LatexPackageImportProvider,
                LatexBeginCommandCompletionProvider,
                LatexCommandSymbolCompletionProvider,
                LatexTikzCommandCompletionProvider,
                LatexKernelCommandCompletionProvider,
                LatexUserCommandCompletionProvider
            )
        })
        .await;

        items
            .into_iter()
            .unique_by(|item| item.label.clone())
            .take(COMPLETION_LIMIT)
            .collect()
    }
}
