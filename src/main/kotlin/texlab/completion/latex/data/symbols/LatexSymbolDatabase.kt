package texlab.completion.latex.data.symbols

import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue

data class LatexSymbolDatabase(val commands: List<LatexCommandSymbol>,
                               val arguments: List<LatexArgumentSymbol>) {
    companion object {
        val INSTANCE = jacksonObjectMapper()
                .readValue<LatexSymbolDatabase>(
                        LatexSymbolDatabase::class.java.getResourceAsStream("symbols.json"))
    }
}
