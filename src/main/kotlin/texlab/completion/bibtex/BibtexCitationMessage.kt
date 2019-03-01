package texlab.completion.bibtex

import kotlinx.coroutines.CompletableDeferred

data class BibtexCitationMessage(val entry: String,
                                 val response: CompletableDeferred<String?>)
