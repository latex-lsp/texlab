package texlab.metadata

import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.overzealous.remark.Remark
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import java.io.IOException
import java.net.URL

class CtanPackageMetadataProvider : MetadataProvider {
    override fun getMetadata(name: String): Metadata? {
        return try {
            val json = URL("https://ctan.org/json/2.0/pkg/$name").readText()
            val mapper = jacksonObjectMapper()
                    .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
            val result = mapper.readValue<Package>(json)
            val description = result.descriptions.firstOrNull { it.language == null }?.text ?: return null
            val documentation = MarkupContent().apply {
                kind = MarkupKind.MARKDOWN
                value = Remark().convert(description)
            }
            Metadata(name, result.caption, documentation)
        } catch (e: IOException) {
            null
        }
    }

    private data class Package(val name: String,
                               val caption: String,
                               val descriptions: List<Description>)

    private data class Description(val language: String?,
                                   val text: String)
}
