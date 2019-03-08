package texlab

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException
import java.net.URI
import java.nio.file.Files
import java.nio.file.InvalidPathException
import java.nio.file.Path
import java.nio.file.Paths
import java.util.*

class Workspace(val documentsByUri: Map<URI, Document> = emptyMap()) {
    fun resolveDocument(uri: URI, relativePath: String): Document? {
        for (target in resolveLinkTargets(uri, relativePath)) {
            val child = File(target).toURI()
            val document = documentsByUri[child]
            if (document != null && document.isFile) {
                return document
            }
        }

        return null
    }

    fun resolveLinkTargets(uri: URI, relativePath: String): List<String> {
        if (uri.scheme != "file") {
            return emptyList()
        }

        val targets = mutableListOf<String>()
        val extensions = arrayOf(".tex", ".sty", ".cls", ".bib")
        return try {
            val basePath = Paths.get(File(uri).parent)
            val fullPath = basePath.resolve(relativePath)
                    .normalize()
                    .toString()
                    .replace('\\', '/')
                    .let { URIHelper.normalizeDriveLetter(it) }
            targets.add(fullPath)
            extensions.forEach { targets.add("$fullPath$it") }
            return targets
        } catch (e: InvalidPathException) {
            emptyList()
        }
    }

    fun relatedDocuments(uri: URI): List<Document> {
        val edges = mutableSetOf<Pair<Document, Document>>()
        documentsByUri.values
                .filterIsInstance<LatexDocument>()
                .filter { it.isFile }
                .forEach { parent ->
                    parent.tree.includes
                            .mapNotNull { resolveDocument(parent.uri, it.path) }
                            .forEach { child ->
                                edges.add(Pair(parent, child))
                                edges.add(Pair(child, parent))
                            }
                }

        val results = mutableListOf<Document>()
        val start = documentsByUri[uri] ?: return results
        val visited = mutableSetOf<Document>()
        val stack = Stack<Document>()
        stack.push(start)
        while (!stack.empty()) {
            val current = stack.pop()
            if (!visited.add(current)) {
                continue
            }

            results.add(current)
            documentsByUri.filterValues { edges.contains(Pair(current, it)) }
                    .forEach { stack.push(it.value) }
        }
        return results
    }

    fun findParent(childUri: URI): Document {
        return relatedDocuments(childUri)
                .filterIsInstance<LatexDocument>()
                .firstOrNull { it.tree.isStandalone }
                ?: documentsByUri.getValue(childUri)
    }

    companion object {
        suspend fun load(file: Path): Document? {
            val extension = file.fileName?.toFile()?.extension ?: return null
            val language = getLanguageByExtension(extension) ?: return null
            return try {
                val text = withContext(Dispatchers.IO) {
                    Files.readAllBytes(file).toString(Charsets.UTF_8)
                }
                Document.create(file.toUri(), text, language)
            } catch (e: IOException) {
                // File is locked
                null
            }
        }
    }
}
