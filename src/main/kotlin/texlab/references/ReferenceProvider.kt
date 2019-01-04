package texlab.references

import org.eclipse.lsp4j.Location

interface ReferenceProvider {
    fun getReferences(request: ReferenceRequest): List<Location>?
}

