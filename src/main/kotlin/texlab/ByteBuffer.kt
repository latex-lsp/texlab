package texlab

import java.nio.ByteBuffer

fun ByteBuffer.getString(index: Int): String {
    var byte = this[index]
    var length = 0
    while (byte.toInt() != 0) {
        length++
        byte = this[index + length]
    }

    return String(this.array(), index, length, Charsets.US_ASCII)
}
