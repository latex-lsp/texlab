package texlab.completion.latex.data

import texlab.Document
import texlab.LatexDocument

interface LatexComponentSource {
    fun getComponent(fileName: String): LatexComponent?

    fun getRelatedComponents(component: LatexComponent): List<LatexComponent> {
        return component.references
                .mapNotNull { getComponent(it) }
                .plus(component)
    }

    fun getRelatedComponents(documents: List<Document>): List<LatexComponent> {
        return documents.filterIsInstance<LatexDocument>()
                .flatMap { it.tree.components }
                .mapNotNull { getComponent(it) }
                .flatMap { getRelatedComponents(it) }
                .distinct()
    }
}
