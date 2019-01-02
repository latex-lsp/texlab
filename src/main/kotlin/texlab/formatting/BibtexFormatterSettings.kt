package texlab.formatting

import java.util.*

data class BibtexFormatterSettings(val insertSpaces: Boolean,
                                   val tabSize: Int,
                                   val style: BibtexStyle) {
    val indent: String = if (insertSpaces) {
        Collections.nCopies(tabSize, " ").joinToString("")
    } else {
        "\t"
    }
}

