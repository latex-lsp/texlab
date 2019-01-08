package texlab

import org.eclipse.lsp4j.jsonrpc.services.JsonNotification
import org.eclipse.lsp4j.services.LanguageClient

interface CustomLanguageClient : LanguageClient {
    @JsonNotification("window/setStatus")
    fun setStatus(statusParams: StatusParams)
}
