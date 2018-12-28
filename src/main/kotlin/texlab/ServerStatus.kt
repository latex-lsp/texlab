package texlab

enum class ServerStatus(val value: Int) {
    IDLE(0),
    BUILDING(1),
    INDEXING(2),
}
