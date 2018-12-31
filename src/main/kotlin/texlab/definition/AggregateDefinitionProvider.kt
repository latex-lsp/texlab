package texlab.definition

import org.eclipse.lsp4j.Location

class AggregateDefinitionProvider(private vararg val providers: DefinitionProvider) : DefinitionProvider {
    override fun find(request: DefinitionRequest): Location? {
        for (provider in providers) {
            val location = provider.find(request)
            if (location != null) {
                return location
            }
        }
        return null
    }
}
