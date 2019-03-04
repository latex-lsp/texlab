package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import kotlin.streams.toList

class IncludeGraphicsProvider(val root: Path?) : LatexArgumentProvider() {
    override val commandNames: List<String> = listOf("\\includegraphics")

    override val argumentIndex: Int = 0

    override fun complete(request: FeatureRequest<CompletionParams>, command: LatexCommandSyntax)
            : List<CompletionItem> {
        if (request.uri.scheme != "file") {
            return emptyList()
        }

        val directory = Paths.get(request.uri).parent
        val matcher = FileSystems.getDefault().getPathMatcher("glob:**.{pdf,png,jpg,jpeg,bmp}")
        return Files.walk(root ?: directory)
                .filter { Files.isRegularFile(it) }
                .filter { matcher.matches(it) }
                .map { relativize(directory, it) }
                .map { CompletionItemFactory.createFile(it) }
                .toList()
    }

    private fun relativize(base: Path, relative: Path): String {
        return base.relativize(relative)
                .toString()
                .replace('\\', '/')
    }
}
