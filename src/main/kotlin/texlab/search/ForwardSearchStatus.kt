package texlab.search

enum class ForwardSearchStatus(val value: Int) {
    SUCCESS(0),
    ERROR(1),
    FAILURE(2),
    UNCONFIGURED(3);
}

