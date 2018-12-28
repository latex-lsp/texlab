package texlab.build

import java.io.File
import java.net.URI
import java.nio.file.InvalidPathException
import java.nio.file.Paths
import java.util.regex.Matcher
import java.util.regex.Pattern

object BuildErrorParser {
    private const val MAX_LINE_LENGTH = 79

    private val PACKAGE_MESSAGE_REGEX = """^\([a-zA-Z_\-]+\)\s*(?<Message>.*)$""".toRegex()
    private val FILE_REGEX = """\((?<File>[^\r\n()]+\.(tex|sty|cls))""".toRegex()
    private const val TEX_ERROR_REGEX = """^! ((?<Message1>(.|\r|\n)*?)\r?\nl\.(?<Line>\d+)|(?<Message2>[^\r\n]*))"""
    private const val WARNING_REGEX = """(LaTeX|Package [a-zA-Z_\-]+) Warning: (?<Message>[^\r\n]*)"""
    private const val BAD_BOX_REGEX = """(?<Message>(Ov|Und)erfull \\[hv]box[^\r\n]*lines? (?<Line>\d+)[^\r\n]*)"""

    fun parse(parent: URI, log: String): List<BuildError> {
        val newLog = prepareLog(log)
        val ranges = FILE_REGEX.findAll(newLog)
                .mapNotNull { getFileRange(parent, newLog, it) }
                .sortedBy { it.length }
                .toList()

        fun parse(regex: String, flags: Int, kind: BuildErrorKind): Sequence<BuildError> = sequence {
            val pattern = Pattern.compile(regex, flags)
            val matcher = pattern.matcher(newLog)
            while (matcher.find()) {
                val messageGroup = matcher.tryGroup("Message")
                        ?: matcher.tryGroup("Message1")
                        ?: matcher.tryGroup("Message2")
                val message = messageGroup!!.lines()[0]

                val file = ranges.first { it.contains(matcher.start()) }.uri
                val line = if (file == null) {
                    null
                } else {
                    matcher.tryGroup("Line")?.toInt()?.let { it - 1 }
                }
                yield(BuildError(file ?: parent, kind, message, line))
            }
        }

        val texErrors = parse(TEX_ERROR_REGEX, Pattern.MULTILINE, BuildErrorKind.ERROR)
        val warnings = parse(WARNING_REGEX, 0, BuildErrorKind.WARNING)
        val badBoxes = parse(BAD_BOX_REGEX, 0, BuildErrorKind.WARNING)
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
                    newLines[newLines.size - 1] += " " + match.groups[1]!!.value
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

    private fun getFileRange(parent: URI, log: String, match: MatchResult): FileRange? {
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

        return try {
            val basePath = Paths.get(File(parent).parent)
            val fullPath = basePath.resolve(match.groups[1]!!.value).normalize()
            val uri = if (fullPath.startsWith(basePath)) {
                fullPath.toUri()
            } else {
                null
            }
            FileRange(uri, match.range.start, end)
        } catch (e: InvalidPathException) {
            null
        }
    }

    private fun Matcher.tryGroup(name: String): String? {
        return try {
            return group(name)
        } catch (e: IllegalArgumentException) {
            null
        }
    }
}
