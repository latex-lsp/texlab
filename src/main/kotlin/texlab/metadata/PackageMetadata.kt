package texlab.metadata

import org.eclipse.lsp4j.MarkupContent

data class PackageMetadata(val name: String,
                           val caption: String,
                           private val descriptions: List<PackageDescription>) {
    val description: MarkupContent?
        get() = descriptions.firstOrNull() { it.language == null }?.markup
}
