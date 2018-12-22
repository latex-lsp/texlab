package texlab.syntax

interface TokenSource<T> {
    fun next(): T?
}
