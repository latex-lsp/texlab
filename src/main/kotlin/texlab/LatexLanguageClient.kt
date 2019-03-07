package texlab

import org.eclipse.lsp4j.jsonrpc.services.JsonNotification
import org.eclipse.lsp4j.services.LanguageClient

interface LatexLanguageClient : LanguageClient {
    @JsonNotification("window/progress")
    fun progress(params: ProgressParams)
}
