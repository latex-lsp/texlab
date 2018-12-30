package texlab

import java.io.File
import java.net.URI
import java.nio.file.InvalidPathException
import java.nio.file.Paths
import java.util.*
import javax.swing.JOptionPane

class Workspace {
    val documents = mutableListOf<Document>()

    fun resolve(uri: URI, relativePath: String): Document? {
        fun find(path: String): Document? {
            val child = File(path).toURI()
            return documents
                    .filter { it.isFile }
                    .firstOrNull { it.uri == child }
        }

        if (uri.scheme != "file") {
            return null
        }

        val extensions = arrayOf(".tex", ".sty", ".cls", ".bib")
        return try {
            val basePath = Paths.get(File(uri).parent)
            val fullPath = basePath.resolve(relativePath).toString().replace('\\', '/')
            var document = find(fullPath)
            extensions.forEach { document = document ?: find("$fullPath$it") }
            return document
        } catch (e: InvalidPathException) {
            null
        }
    }

    fun relatedDocuments(uri: URI): List<Document> {
        val edges = mutableSetOf<Pair<Document, Document>>()
        documents.filterIsInstance<LatexDocument>()
                .filter { it.isFile }
                .forEach { parent ->
                    parent.tree.includes
                            .mapNotNull { resolve(parent.uri, it.path) }
                            .forEach { child ->
                                edges.add(Pair(parent, child))
                                edges.add(Pair(child, parent))
                            }
                }

        val results = mutableListOf<Document>()
        val start = documents.firstOrNull { it.uri == uri } ?: return results
        val visited = mutableSetOf<Document>()
        val stack = Stack<Document>()
        stack.push(start)
        while (!stack.empty()) {
            val current = stack.pop()
            if (!visited.add(current)) {
                continue
            }

            results.add(current)
            documents.filter { edges.contains(Pair(current, it)) }
                    .forEach { stack.push(it) }
        }
        return results
    }
}
