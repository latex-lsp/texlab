package texlab

import org.eclipse.lsp4j.TextDocumentPositionParams
import org.eclipse.lsp4j.jsonrpc.services.JsonRequest
import org.eclipse.lsp4j.jsonrpc.services.JsonSegment
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.build.BuildParams
import texlab.build.BuildStatus
import texlab.forwardSearch.ForwardSearchStatus
import java.util.concurrent.CompletableFuture

@JsonSegment("textDocument")
interface CustomTextDocumentService : TextDocumentService {
    @JsonRequest
    fun build(params: BuildParams): CompletableFuture<BuildStatus>

    @JsonRequest
    fun forwardSearch(params: TextDocumentPositionParams): CompletableFuture<ForwardSearchStatus>
}
