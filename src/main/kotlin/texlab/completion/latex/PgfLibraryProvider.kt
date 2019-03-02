package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import org.eclipse.lsp4j.CompletionParams
import texlab.completion.CompletionItemFactory
import texlab.provider.FeatureRequest
import texlab.syntax.latex.LatexCommandSyntax

object PgfLibraryProvider : LatexArgumentProvider() {
    private val libraries = arrayOf(
            "arrows",
            "arrows.meta",
            "arrows.spaced",
            "curvilinear",
            "datavisualization.barcharts",
            "datavisualization.formats.functions",
            "datavisualization.polar",
            "decorations.footprints",
            "decorations.fractals",
            "decorations.markings",
            "decorations.pathmorphing",
            "decorations.pathreplacing",
            "decorations.shapes",
            "decorations.text",
            "fadings",
            "fixedpointarithmetic",
            "fpu",
            "intersections",
            "lindenmayersystems",
            "luamath",
            "patterns",
            "patterns.meta",
            "plothandlers",
            "plotmarks",
            "profiler",
            "shadings",
            "shapes.arrows",
            "shapes.callouts",
            "shapes",
            "shapes.gates.ee",
            "shapes.gates.ee.IEC",
            "shapes.gates.logic",
            "shapes.gates.logic.IEC",
            "shapes.gates.logic.US",
            "shapes.geometric",
            "shapes.misc",
            "shapes.multipart",
            "shapes.symbols",
            "snakes",
            "svg.path"
    )

    private val items = libraries.map { CompletionItemFactory.createPgfLibrary(it) }

    override val commandNames: List<String> = listOf("\\usepgflibrary")

    override val argumentIndex: Int = 0

    override fun complete(request: FeatureRequest<CompletionParams>,
                          command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}

