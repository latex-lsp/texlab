package texlab

import org.eclipse.lsp4j.jsonrpc.services.JsonRequest
import org.eclipse.lsp4j.jsonrpc.services.JsonSegment
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.build.BuildParams
import texlab.build.BuildStatus
import java.util.concurrent.CompletableFuture

@JsonSegment("textDocument")
interface CustomTextDocumentService : TextDocumentService {
    @JsonRequest
    fun build(params: BuildParams): CompletableFuture<BuildStatus>
}
