package texlab.definition

import org.eclipse.lsp4j.Location

interface DefinitionProvider {
    fun find(request: DefinitionRequest): Location?
}

