package texlab.metadata

import com.fasterxml.jackson.databind.util.StdConverter
import com.overzealous.remark.Remark
import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind

class HtmlToMarkdownConverter : StdConverter<String, MarkupContent>() {
    override fun convert(value: String): MarkupContent {
        return MarkupContent().apply {
            kind = MarkupKind.MARKDOWN
            this.value = Remark().convert(value)
        }
    }
}
