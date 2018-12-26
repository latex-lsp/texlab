package texlab.build

import java.net.URI

data class FileRange(val uri: URI?, val start: Int, val end: Int) {
    val length: Int = end - start + 1

    fun contains(index: Int): Boolean = index in start..end
}
