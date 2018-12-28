package texlab.completion.latex.data

import java.util.*
import kotlin.math.min

object ComponentFinder {
    fun <T> find(vertices: Iterable<T>, getNeighbours: (vertex: T) -> Iterable<T>): List<List<T>> {
        val nodesByVertex = vertices.associate { Pair(it, Node(it)) }
        var index = 0
        val stack = Stack<Node<T>>()
        val components = mutableListOf<MutableList<T>>()

        fun processNode(node: Node<T>) {
            node.index = index
            node.lowLink = index
            index++
            stack.push(node)
            node.onStack = true

            val neighbours = getNeighbours(node.value).map { nodesByVertex.getValue(it) }
            for (neighbour in neighbours) {
                if (neighbour.index == -1) {
                    processNode(neighbour)
                    node.lowLink = min(node.lowLink, neighbour.lowLink)
                } else if (neighbour.onStack) {
                    node.lowLink = min(node.lowLink, neighbour.index)
                }
            }

            if (node.lowLink == node.index) {
                val component = mutableListOf<T>()
                var next: Node<T>
                do {
                    next = stack.pop()
                    next.onStack = false
                    component.add(next.value)
                } while (next != node)
                components.add(component)
            }
        }

        for (node in nodesByVertex.values) {
            if (node.index == -1) {
                processNode(node)
            }
        }

        return components
    }

    private class Node<T>(val value: T,
                          var index: Int = -1,
                          var lowLink: Int = -1,
                          var onStack: Boolean = false)
}
