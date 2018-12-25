package texlab

import org.eclipse.lsp4j.DidChangeConfigurationParams
import org.eclipse.lsp4j.DidChangeWatchedFilesParams
import org.eclipse.lsp4j.services.WorkspaceService

class WorkspaceServiceImpl : WorkspaceService {
    override fun didChangeWatchedFiles(params: DidChangeWatchedFilesParams) {
    }

    override fun didChangeConfiguration(params: DidChangeConfigurationParams) {
    }
}
