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
            if (Files.exists(directory)) {
                val mapper = jacksonObjectMapper()
                val index = mapper.readValue<LatexSymbolIndex>(directory.resolve("index.json").toFile())
                return LatexSymbolDatabase(index, directory)
            }

            val stream = LatexSymbolDatabase::class.java.getResourceAsStream("symbols.zip")
            stream.use { ZipUtil.unpack(it, directory.toFile()) }
            return loadOrCreate(directory)
        }
    }
}
