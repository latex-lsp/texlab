package texlab

import org.eclipse.lsp4j.TextDocumentPositionParams
import org.eclipse.lsp4j.jsonrpc.services.JsonRequest
import org.eclipse.lsp4j.jsonrpc.services.JsonSegment
import org.eclipse.lsp4j.services.TextDocumentService
import texlab.build.BuildParams
import texlab.build.BuildResult
import texlab.search.ForwardSearchResult
import java.util.concurrent.CompletableFuture

@JsonSegment("textDocument")
interface LatexTextDocumentService : TextDocumentService {
    @JsonRequest
    fun build(params: BuildParams): CompletableFuture<BuildResult>

    @JsonRequest
    fun forwardSearch(params: TextDocumentPositionParams): CompletableFuture<ForwardSearchResult>
}
