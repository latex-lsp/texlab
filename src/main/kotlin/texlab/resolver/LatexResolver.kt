package texlab.resolver

import texlab.getString
import java.io.File
import java.io.IOException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths


private const val TEXLIVE_DATABASE_PATH = "ls-R"
private const val MIKTEX_DATABASE_PATH = "miktex/data/le"

class LatexResolver(val filesByName: Map<String, File>) {
    companion object {
        fun empty(): LatexResolver {
            return LatexResolver(emptyMap())
        }

        fun create(): LatexResolver {
            try {
                val rootDirectories = findRootDirectories()
                println(rootDirectories)
                val kind = detectDistribution(rootDirectories)
                if (kind == LatexDistributionKind.UNKNOWN) {
                    val error = TexDistributionError.UNKNOWN_DISTRIBUTION
                    throw InvalidTexDistributionException(error)
                }

                return LatexResolver(readDatabase(rootDirectories, kind))
            } catch (e: IOException) {
                val error = TexDistributionError.INVALID_DISTRIBUTION
                throw InvalidTexDistributionException(error)
            }
        }

        private fun findRootDirectories(): List<Path> {
            try {
                val texmf = runKpsewhich("-var-value", "TEXMF")
                return runKpsewhich("--expand-braces=$texmf")
                        .split(';')
                        .map { Paths.get(it.replace("!", "")) }
                        .filter { Files.exists(it) }
                        .distinct()
            } catch (e: IOException) {
                val error = TexDistributionError.KPSEWHICH_NOT_FOUND
                throw InvalidTexDistributionException(error)
            }
        }

        private fun runKpsewhich(vararg args: String): String {
            val process = ProcessBuilder("kpsewhich", *args)
                    .redirectOutput(ProcessBuilder.Redirect.PIPE)
                    .start()
            process.waitFor()
            return process.inputStream.bufferedReader().readLine()
        }

        private fun detectDistribution(directories: List<Path>): LatexDistributionKind {
            val kinds = directories.map {
                when {
                    Files.exists(it.resolve(TEXLIVE_DATABASE_PATH)) ->
                        LatexDistributionKind.TEXLIVE
                    Files.exists(it.resolve(MIKTEX_DATABASE_PATH)) ->
                        LatexDistributionKind.MIKTEX
                    else ->
                        LatexDistributionKind.UNKNOWN
                }
            }

            return kinds.firstOrNull { it != LatexDistributionKind.UNKNOWN }
                    ?: LatexDistributionKind.UNKNOWN
        }

        private fun readDatabase(rootDirectories: List<Path>, kind: LatexDistributionKind)
                : Map<String, File> {
            val filesByName = mutableMapOf<String, File>()
            for (directory in rootDirectories) {
                val database = when (kind) {
                    LatexDistributionKind.TEXLIVE -> {
                        val file = directory.resolve(TEXLIVE_DATABASE_PATH)
                        if (Files.exists(file)) {
                            val lines = Files.readAllLines(file)
                            parseTexliveDatabase(directory, lines)
                        } else {
                            emptySequence()
                        }
                    }
                    LatexDistributionKind.MIKTEX -> {
                        directory.resolve(MIKTEX_DATABASE_PATH)
                                .toFile()
                                .listFiles()
                                .asSequence()
                                .filter { it.extension.matches(Regex("""fndb-\d+""")) }
                                .map { ByteBuffer.wrap(Files.readAllBytes(it.toPath())) }
                                .map { it.order(ByteOrder.LITTLE_ENDIAN) }
                                .flatMap { parseMiktexDatabase(it) }
                    }
                    LatexDistributionKind.UNKNOWN -> {
                        val error = TexDistributionError.UNKNOWN_DISTRIBUTION
                        throw InvalidTexDistributionException(error)
                    }
                }

                filesByName.putAll(database.associateBy { it.name })
            }
            return filesByName
        }

        private fun parseTexliveDatabase(rootDirectory: Path, lines: Iterable<String>)
                : Sequence<File> = sequence {
            var currentDirectory = Paths.get("")
            for (line in lines.filter { it.isNotBlank() && !it.startsWith("%") }) {
                if (line.endsWith(":")) {
                    val path = line.substring(0, line.length - 1)
                    currentDirectory = rootDirectory.resolve(path).normalize()
                } else {
                    val file = currentDirectory.resolve(line).toFile()
                    if (file.extension.isNotBlank()) {
                        yield(file)
                    }
                }
            }
        }

        private fun parseMiktexDatabase(buffer: ByteBuffer): Sequence<File> = sequence {
            if (buffer.getInt(0) != 0x42444e46) { // signature of fndb file
                val error = TexDistributionError.INVALID_DISTRIBUTION
                throw InvalidTexDistributionException(error)
            }

            val tableAddress = buffer.getInt(4 * 4) // pointer to first record
            val tableSize = buffer.getInt(6 * 4) // number of files (records)

            for (i in 0 until tableSize) {
                val offset = tableAddress + i * 16
                val fileName = buffer.getString(buffer.getInt(offset))
                val directory = buffer.getString(buffer.getInt(offset + 4))
                val file = Paths.get(directory, fileName).toFile()
                yield(file)
            }
        }
    }
}
