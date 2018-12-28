package texlab.build

interface BuildListener {
    fun stdout(line: String)

    fun stderr(line: String)
}
