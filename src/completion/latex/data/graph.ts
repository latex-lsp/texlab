interface Node<T> {
  value: T;
  index: number;
  lowLink: number;
  onStack: boolean;
}

export function stronglyConnectedComponents<T>(
  vertices: T[],
  getNeighbours: (vertex: T) => T[],
): T[][] {
  const nodesByVertex = new Map<T, Node<T>>();
  vertices.forEach(x => {
    const node: Node<T> = {
      value: x,
      index: -1,
      lowLink: -1,
      onStack: false,
    };

    nodesByVertex.set(x, node);
  });

  const stack: Array<Node<T>> = [];
  const components: T[][] = [];
  let index = 0;

  function processNode(node: Node<T>) {
    node.index = index;
    node.lowLink = index;
    index++;
    stack.push(node);
    node.onStack = true;

    const neighbours = getNeighbours(node.value).map(
      x => nodesByVertex.get(x)!,
    );

    for (const neighbour of neighbours) {
      if (neighbour.index === -1) {
        processNode(neighbour);
        node.lowLink = Math.min(node.lowLink, neighbour.lowLink);
      } else if (neighbour.onStack) {
        node.lowLink = Math.min(node.lowLink, neighbour.index);
      }
    }

    if (node.lowLink === node.index) {
      const component: T[] = [];
      let next: Node<T>;
      do {
        next = stack.pop()!;
        next.onStack = false;
        component.push(next.value);
      } while (next !== node);

      components.push(component);
    }
  }

  for (const node of nodesByVertex.values()) {
    if (node.index === -1) {
      processNode(node);
    }
  }

  return components;
}
