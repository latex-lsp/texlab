package texlab.completion.latex

import org.eclipse.lsp4j.CompletionItem
import texlab.completion.CompletionItemFactory
import texlab.completion.CompletionRequest
import texlab.syntax.latex.LatexCommandSyntax

object LatexColorProvider : LatexArgumentProvider() {

    private val colors: Array<String> = arrayOf(
            "black",
            "blue",
            "brown",
            "cyan",
            "darkgray",
            "gray",
            "green",
            "lightgray",
            "lime",
            "magenta",
            "olive",
            "orange",
            "pink",
            "purple",
            "red",
            "teal",
            "violet",
            "white",
            "yellow",
            "Apricot",
            "Bittersweet",
            "Blue",
            "BlueViolet",
            "Brown",
            "CadetBlue",
            "Cerulean",
            "Cyan",
            "DarkOrchid",
            "ForestGreen",
            "Goldenrod",
            "Green",
            "JungleGreen",
            "LimeGreen",
            "Mahogany",
            "Melon",
            "Mulberry",
            "OliveGreen",
            "OrangeRed",
            "Peach",
            "PineGreen",
            "ProcessBlue",
            "RawSienna",
            "RedOrange",
            "Rhodamine",
            "RoyalPurple",
            "Salmon",
            "Sepia",
            "SpringGreen",
            "TealBlue",
            "Turquoise",
            "VioletRed",
            "WildStrawberry",
            "YellowGreen",
            "Aquamarine",
            "Black",
            "BlueGreen",
            "BrickRed",
            "BurntOrange",
            "CarnationPink",
            "CornflowerBlue",
            "Dandelion",
            "Emerald",
            "Fuchsia",
            "Gray",
            "GreenYellow",
            "Lavender",
            "Magenta",
            "Maroon",
            "MidnightBlue",
            "NavyBlue",
            "Orange",
            "Orchid",
            "Periwinkle",
            "Plum",
            "Purple",
            "Red",
            "RedViolet",
            "RoyalBlue",
            "RubineRed",
            "SeaGreen",
            "SkyBlue",
            "Tan",
            "Thistle",
            "Violet",
            "White",
            "Yellow",
            "YellowOrange")

    private val items: List<CompletionItem> = colors.map { CompletionItemFactory.createColor(it) }

    override val commandNames: List<String> = listOf(
            "\\color",
            "\\colorbox",
            "\\textcolor",
            "\\pagecolor",
            "\\colorlet",
            "\\definespotcolor")

    override val argumentIndex: Int = 0

    override fun complete(request: CompletionRequest, command: LatexCommandSyntax): List<CompletionItem> {
        return items
    }
}
