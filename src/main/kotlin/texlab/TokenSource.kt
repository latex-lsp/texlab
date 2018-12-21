package texlab

interface TokenSource<T> {
    fun next(): T?
}
