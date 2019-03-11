import { BibtexSyntaxTree } from './analysis';

describe('BibTeX Analysis', () => {
  it('should parse entries', () => {
    const tree = new BibtexSyntaxTree(`@article{foo, }\n@article{bar,}`);
    expect(tree.entries).toEqual([
      tree.root.children[0],
      tree.root.children[1],
    ]);
  });
});
