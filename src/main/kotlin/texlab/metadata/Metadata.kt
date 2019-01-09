package texlab.metadata

import org.eclipse.lsp4j.MarkupContent

data class Metadata(val name: String,
                    val detail: String?,
                    val documentation: MarkupContent?)
