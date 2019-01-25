package texlab.completion.latex.data

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.SerializationFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import texlab.ProgressListener
import texlab.ProgressParams
import texlab.completion.latex.KernelPrimitives
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import kotlin.coroutines.CoroutineContext

class LatexComponentDatabase(override val coroutineContext: CoroutineContext,
                             private val mapper: ObjectMapper,
                             private val databaseFile: File,
                             private val resolver: LatexResolver,
                             components: List<LatexComponent>,
                             private val listener: ProgressListener?) : CoroutineScope, LatexComponentSource {
    private val componentsByName = ConcurrentHashMap<String, LatexComponent>()
    private val channel = Channel<File>(Channel.UNLIMITED)

    init {
        mapper.enable(SerializationFeature.INDENT_OUTPUT)

        for (component in components) {
            component.fileNames.forEach { componentsByName[it] = component }
        }

        GlobalScope.launch(Dispatchers.IO) {
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

        val progressParams = ProgressParams("index", "Indexing...", file.name)
        listener?.onReportProgress(progressParams)

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
            listener?.onReportProgress(progressParams.copy(message = unit.file.name))

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

        listener?.onReportProgress(progressParams.copy(done = true))
        save()
    }

    private fun loadRelatedUnits(file: File): Map<File, LatexUnit>? {
        val unit = LatexUnit.load(file, resolver) ?: return null
        return unit.references.filter { !componentsByName.containsKey(it.name) }
                .mapNotNull { LatexUnit.load(it, resolver) }
                .plus(unit)
                .associateBy { it.file }
    }

    private fun save() {
        mapper.writeValue(databaseFile, componentsByName.values)
    }

    companion object {
        fun loadOrCreate(coroutineContext: CoroutineContext,
                         databaseFile: File,
                         resolver: LatexResolver,
                         listener: ProgressListener?): LatexComponentDatabase {
            val mapper = jacksonObjectMapper()
            val components = if (databaseFile.exists()) {
                mapper.readValue<List<LatexComponent>>(databaseFile)
            } else {
                emptyList()
            }

            return LatexComponentDatabase(coroutineContext, mapper, databaseFile, resolver, components, listener)
        }
    }
}
