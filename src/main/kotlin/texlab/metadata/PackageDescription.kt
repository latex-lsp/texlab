package texlab.metadata

import com.fasterxml.jackson.annotation.JsonProperty
import com.fasterxml.jackson.databind.annotation.JsonDeserialize
import org.eclipse.lsp4j.MarkupContent

data class PackageDescription(val language: String?,
                              @JsonDeserialize(converter = HtmlToMarkdownConverter::class)
                              @JsonProperty("text") val markup: MarkupContent)
