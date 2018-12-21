package texlab

class TokenBuffer<T>(private val source: TokenSource<T>) {
    private val buffer = mutableListOf<T>()

    val available: Boolean
        get() = peek() != null

    fun peek(lookAhead: Int = 0): T? {
        while (buffer.size < lookAhead + 1) {
            val token = source.next() ?: return null
            buffer.add(token)
        }
        return buffer[lookAhead]
    }

    fun next(): T {
        val token = peek()
        buffer.removeAt(0)
        return token!!
    }
}
