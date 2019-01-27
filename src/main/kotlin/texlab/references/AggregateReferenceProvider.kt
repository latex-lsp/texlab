package texlab.references

import org.eclipse.lsp4j.Location

class AggregateReferenceProvider(private vararg val providers: ReferenceProvider) : ReferenceProvider {
    override fun getReferences(request: ReferenceRequest): List<Location>? {
        for (provider in providers) {
            val references = provider.getReferences(request)
            if (references != null) {
                return references
            }
        }
        return null
    }
}
