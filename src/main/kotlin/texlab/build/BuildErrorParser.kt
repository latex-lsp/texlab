package texlab.build

import java.io.File
import java.net.URI
import java.nio.file.Paths

object BuildErrorParser {
    private const val MAX_LINE_LENGTH = 79

    private val PACKAGE_MESSAGE_REGEX = """^\([a-zA-Z_\-]+\)\s*(?<Message>.*)$""".toRegex()
    private val FILE_REGEX = """\((?<File>[^\r\n()]+\.(tex|sty|cls))""".toRegex()
    private val TEX_ERROR_REGEX = """^! ((?<Message1>(.|\r|\n)*?)\r?\nl\.(?<Line>\d+)|(?<Message2>[^\r\n]*))""".toRegex(RegexOption.MULTILINE)
    private val WARNING_REGEX = """(LaTeX|Package [a-zA-Z_\-]+) Warning: (?<Message>[^\r\n]*)""".toRegex()
    private val BAD_BOX_REGEX = """(?<Message>(Ov|Und)erfull \\[hv]box[^\r\n]*lines? (?<Line>\d+)[^\r\n]*)""".toRegex()

    fun parse(parent: URI, log: String): List<BuildError> {
        val newLog = prepareLog(log)
        val ranges = FILE_REGEX.findAll(newLog)
                .map { getFileRange(parent, newLog, it) }
                .sortedBy { it.length }
                .toList()

        fun parse(regex: Regex, kind: BuildErrorKind): Sequence<BuildError> = sequence {
            for (match in regex.findAll(newLog)) {
                val messageGroup = match.group("Message")
                        ?: match.group("Message1")
                        ?: match.group("Message2")
                val message = messageGroup!!.value.lines()[0]

                val file = ranges.first { it.contains(match.range.start) }.uri
                val line = if (file == null) {
                    null
                } else {
                    match.group("Line")?.value?.toInt()?.let { it - 1 }
                }
                yield(BuildError(file ?: parent, kind, message, line))
            }
        }

        val texErrors = parse(TEX_ERROR_REGEX, BuildErrorKind.ERROR)
        val warnings = parse(WARNING_REGEX, BuildErrorKind.WARNING)
        val badBoxes = parse(BAD_BOX_REGEX, BuildErrorKind.WARNING)
        return texErrors.plus(warnings)
                .plus(badBoxes)
                .toList()
    }

    private fun prepareLog(log: String): String {
        val oldLines = log.lines()
        val newLines = mutableListOf<String>()
        var index = 0
        while (index < oldLines.size) {
            val line = oldLines[index]
            val match = PACKAGE_MESSAGE_REGEX.matchEntire(line)
            when {
                match != null -> {
                    newLines[newLines.size - 1] += " " + match.groups["Message"]!!.value
                }
                line.endsWith("...") -> {
                    newLines.add(line.substring(0, line.length - 3))
                    newLines[newLines.size - 1] += oldLines[index++]
                }
                line.length == MAX_LINE_LENGTH -> {
                    newLines.add(line)
                    newLines[newLines.size - 1] += oldLines[index++]
                }
                else -> {
                    newLines.add(line)
                }
            }
            index++
        }
        return newLines.joinToString(System.lineSeparator())
    }

    private fun getFileRange(parent: URI, log: String, match: MatchResult): FileRange {
        var balance = 1
        var end = match.range.first + 1
        while (balance > 0 && end < log.length) {
            if (log[end] == '(') {
                balance++
            } else if (log[end] == ')') {
                balance--
            }
            end++
        }

        val basePath = Paths.get(File(parent).parent)
        val fullPath = basePath.resolve(match.groups["File"]!!.value).normalize()
        val uri = if (fullPath.startsWith(basePath)) {
            fullPath.toUri()
        } else {
            null
        }

        return FileRange(uri, match.range.start, end)
    }

    private fun MatchResult.group(name: String): MatchGroup? {
        return try {
            groups[name]
        } catch (e: IllegalArgumentException) {
            null
        }
    }
}
