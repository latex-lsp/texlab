import { stronglyConnectedComponents } from './graph';

describe('stronglyConnectedComponents', () => {
  it('should find all strongly connected components', () => {
    const vertices = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    const graph = new Map([
      ['a', ['b']],
      ['b', ['c', 'e', 'f']],
      ['c', ['d', 'g']],
      ['d', ['c', 'h']],
      ['e', ['a', 'f']],
      ['f', ['g']],
      ['g', ['f']],
      ['h', ['d']],
    ]);

    const components = stronglyConnectedComponents(
      vertices,
      x => graph.get(x)!,
    );

    expect(components).toEqual([['f', 'g'], ['h', 'd', 'c'], ['e', 'b', 'a']]);
  });
});
