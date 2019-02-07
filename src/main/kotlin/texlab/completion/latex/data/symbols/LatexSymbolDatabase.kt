package texlab.completion.latex.data.symbols

import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import org.zeroturnaround.zip.ZipUtil
import java.nio.file.Files
import java.nio.file.Path

data class LatexSymbolDatabase(val index: LatexSymbolIndex, val directory: Path) {
    fun resolve(imageId: Int): Path = directory.resolve("$imageId.png")

    companion object {
        fun loadOrCreate(directory: Path): LatexSymbolDatabase {
            val indexFile = directory.resolve("index.json")
            if (Files.exists(indexFile)) {
                val mapper = jacksonObjectMapper()
                val index = mapper.readValue<LatexSymbolIndex>(indexFile.toFile())
                return LatexSymbolDatabase(index, directory)
            }

            val stream = LatexSymbolDatabase::class.java.getResourceAsStream("symbols.zip")
            stream.use { ZipUtil.unpack(it, directory.toFile()) }
            return loadOrCreate(directory)
        }
    }
}
