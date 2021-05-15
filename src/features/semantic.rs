mod bibtex;
pub mod legend;

use cancellation::CancellationToken;
use lsp_types::{SemanticTokens, SemanticTokensRangeParams};

use self::bibtex::find_bibtex_semantic_tokens_range;

use super::FeatureRequest;

pub fn find_semantic_tokens_range(
    request: FeatureRequest<SemanticTokensRangeParams>,
    cancellation_token: &CancellationToken,
) -> Option<SemanticTokens> {
    let mut data = Vec::new();
    find_bibtex_semantic_tokens_range(&request, &mut data, cancellation_token);
    Some(SemanticTokens {
        result_id: None,
        data,
    })
}
