package texlab

import java.util.*

data class BibtexFormatterSettings(val insertSpaces: Boolean, val tabSize: Int, val lineLength: Int) {
    val indent: String = if (insertSpaces) {
        Collections.nCopies(tabSize, " ").joinToString("")
    } else {
        "\t"
    }
}
