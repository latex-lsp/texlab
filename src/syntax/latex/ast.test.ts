import {
  descendants,
  LatexCommandSyntax,
  LatexDocumentSyntax,
  LatexGroupSyntax,
  LatexTextSyntax,
} from './ast';
import { LatexToken, LatexTokenKind } from './lexer';

describe('LaTeX AST', () => {
  it('should be able to flatten the tree', () => {
    const child1 = new LatexCommandSyntax(
      new LatexToken(
        { line: 0, character: 0 },
        '\\foo',
        LatexTokenKind.Command,
      ),
      undefined,
      [],
    );
    const child3 = new LatexTextSyntax([
      new LatexToken({ line: 1, character: 0 }, 'foo', LatexTokenKind.Word),
    ]);
    const child2 = new LatexGroupSyntax(
      new LatexToken({ line: 2, character: 0 }, '{', LatexTokenKind.BeginGroup),
      [child3],
      undefined,
    );
    const root = new LatexDocumentSyntax([child1, child2]);
    expect(descendants(root)).toEqual([root, child1, child2, child3]);
  });
});
