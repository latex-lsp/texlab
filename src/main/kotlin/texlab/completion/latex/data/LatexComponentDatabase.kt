package texlab.completion.latex.data

import com.fasterxml.jackson.databind.SerializationFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import texlab.completion.latex.KernelPrimitives
import java.io.File
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.ConcurrentHashMap

class LatexComponentDatabase(private val databaseFile: Path,
                             private val resolver: LatexResolver,
                             components: List<LatexComponent>,
                             private val listener: LatexComponentDatabaseListener?) : LatexComponentSource {
    private val componentsByName = ConcurrentHashMap<String, LatexComponent>()
    private val channel = Channel<File>(Channel.UNLIMITED)

    init {
        for (component in components) {
            component.fileNames.forEach { componentsByName[it] = component }
        }

        GlobalScope.launch {
            for (file in channel) {
                analyze(file)
            }
        }
    }

    override fun getComponent(fileName: String): LatexComponent? {
        val component = componentsByName[fileName]
        if (component != null) {
            return component
        }

        resolver.filesByName[fileName]?.also { channel.offer(it) }
        return null
    }

    private fun analyze(file: File) {
        if (componentsByName.containsKey(file.name)) {
            return
        }

        listener?.onStartIndexing(file)
        val unitsByFile = loadRelatedUnits(file)
        if (unitsByFile == null) {
            componentsByName[file.name] =
                    LatexComponent(listOf(file.name), emptyList(), emptyList(), emptyList())
            save()
            return
        }

        val components = ComponentFinder.find(unitsByFile.values) { unit ->
            unit.references.mapNotNull { unitsByFile[it] }
        }

        for (units in components) {
            val unit = units.first()
            listener?.onStartIndexing(unit.file)

            val candidates = unit.likelyPrimitives.toMutableSet()
            candidates.removeAll(KernelPrimitives.COMMANDS)
            candidates.removeAll(KernelPrimitives.ENVIRONMENTS)
            val references = unit.references.map { it.name }
            for (reference in references.mapNotNull { componentsByName[it] }) {
                candidates.removeAll(reference.commands)
                candidates.removeAll(reference.environments)
            }

            val names = units.map { it.file.name }
            val (commands, environments) = unit.checkPrimitives(candidates)
            val component = LatexComponent(names, references, commands, environments)
            names.forEach { componentsByName[it] = component }
        }

        save()
        listener?.onStopIndexing()
    }

    private fun loadRelatedUnits(file: File): Map<File, LatexUnit>? {
        val unit = LatexUnit.load(file, resolver) ?: return null
        return unit.references.filter { !componentsByName.containsKey(it.name) }
                .mapNotNull { LatexUnit.load(it, resolver) }
                .plus(unit)
                .associateBy { it.file }
    }

    private fun save() {
        val mapper = jacksonObjectMapper()
        mapper.enable(SerializationFeature.INDENT_OUTPUT)
        mapper.writeValue(databaseFile.toFile(), componentsByName.values)
    }

    companion object {
        fun loadOrCreate(databaseFile: Path,
                         resolver: LatexResolver,
                         listener: LatexComponentDatabaseListener?): LatexComponentDatabase {
            return if (Files.exists(databaseFile)) {
                val json = Files.readAllBytes(databaseFile).toString(Charsets.UTF_8)
                val mapper = jacksonObjectMapper()
                val components = mapper.readValue<List<LatexComponent>>(json)
                LatexComponentDatabase(databaseFile, resolver, components, listener)
            } else {
                LatexComponentDatabase(databaseFile, resolver, emptyList(), listener)
            }
        }
    }
}

