package texlab.link

import org.eclipse.lsp4j.DocumentLink
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder

class AggregateLinkProviderTests {
    private fun createProvider(vararg links: DocumentLink): LinkProvider {
        return object : LinkProvider {
            override fun getLinks(request: LinkRequest): List<DocumentLink> {
                return links.toList()
            }
        }
    }

    @Test
    fun `it should merge the links`() {
        val link1 = DocumentLink(Range(Position(1, 2), Position(3, 4)))
        val link2 = DocumentLink(Range(Position(3, 4), Position(5, 6)))
        val link3 = DocumentLink(Range(Position(6, 7), Position(8, 9)))
        val provider1 = createProvider(link1, link2)
        val provider2 = createProvider(link3)
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .link("foo.tex")

        val aggregateProvider = AggregateLinkProvider(provider1, provider2)
        val links = aggregateProvider.getLinks(request)
        assertArrayEquals(arrayOf(link1, link2, link3), links.toTypedArray())
    }
}
