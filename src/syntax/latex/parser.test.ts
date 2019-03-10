import {
  LatexCommandSyntax,
  LatexDocumentSyntax,
  LatexGroupSyntax,
  LatexTextSyntax,
} from './ast';
import { LatexToken, LatexTokenKind } from './lexer';
import { parse } from './parser';

describe('LaTeX Parser', () => {
  function token(
    line: number,
    character: number,
    text: string,
    kind: LatexTokenKind,
  ) {
    return new LatexToken({ line, character }, text, kind);
  }

  it('should parse commands without arguments and options', () => {
    const text = '\\foo';
    const tree = new LatexDocumentSyntax([
      new LatexCommandSyntax(
        token(0, 0, '\\foo', LatexTokenKind.Command),
        undefined,
        [],
      ),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse commands with options', () => {
    const text = '\\foo[bar]';
    const tree = new LatexDocumentSyntax([
      new LatexCommandSyntax(
        token(0, 0, '\\foo', LatexTokenKind.Command),
        new LatexGroupSyntax(
          token(0, 4, '[', LatexTokenKind.BeginOptions),
          [new LatexTextSyntax([token(0, 5, 'bar', LatexTokenKind.Word)])],
          token(0, 8, ']', LatexTokenKind.EndOptions),
        ),
        [],
      ),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse commands with empty arguments', () => {
    const text = '\\foo{}';
    const tree = new LatexDocumentSyntax([
      new LatexCommandSyntax(
        token(0, 0, '\\foo', LatexTokenKind.Command),
        undefined,
        [
          new LatexGroupSyntax(
            token(0, 4, '{', LatexTokenKind.BeginGroup),
            [],
            token(0, 5, '}', LatexTokenKind.EndGroup),
          ),
        ],
      ),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse commands with text arguments', () => {
    const text = '\\begin{foo}';
    const tree = new LatexDocumentSyntax([
      new LatexCommandSyntax(
        token(0, 0, '\\begin', LatexTokenKind.Command),
        undefined,
        [
          new LatexGroupSyntax(
            token(0, 6, '{', LatexTokenKind.BeginGroup),
            [new LatexTextSyntax([token(0, 7, 'foo', LatexTokenKind.Word)])],
            token(0, 10, '}', LatexTokenKind.EndGroup),
          ),
        ],
      ),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse text', () => {
    const text = 'foo bar';
    const tree = new LatexDocumentSyntax([
      new LatexTextSyntax([
        token(0, 0, 'foo', LatexTokenKind.Word),
        token(0, 4, 'bar', LatexTokenKind.Word),
      ]),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse brackets as text (1)', () => {
    const text = '[ ]';
    const tree = new LatexDocumentSyntax([
      new LatexTextSyntax([
        token(0, 0, '[', LatexTokenKind.BeginOptions),
        token(0, 2, ']', LatexTokenKind.EndOptions),
      ]),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse brackets as text (2)', () => {
    const text = ']';
    const tree = new LatexDocumentSyntax([
      new LatexTextSyntax([token(0, 0, ']', LatexTokenKind.EndOptions)]),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should ignore unmatched braces', () => {
    const text = '} }';
    const tree = new LatexDocumentSyntax([]);
    expect(parse(text)).toEqual(tree);
  });

  it('should insert missing braces', () => {
    const text = '{';
    const tree = new LatexDocumentSyntax([
      new LatexGroupSyntax(
        token(0, 0, '{', LatexTokenKind.BeginGroup),
        [],
        undefined,
      ),
    ]);
    expect(parse(text)).toEqual(tree);
  });

  it('should parse nested groups', () => {
    const text = '{\n{\n}\n}';
    const tree = new LatexDocumentSyntax([
      new LatexGroupSyntax(
        token(0, 0, '{', LatexTokenKind.BeginGroup),
        [
          new LatexGroupSyntax(
            token(1, 0, '{', LatexTokenKind.BeginGroup),
            [],
            token(2, 0, '}', LatexTokenKind.EndGroup),
          ),
        ],
        token(3, 0, '}', LatexTokenKind.EndGroup),
      ),
    ]);
    expect(parse(text)).toEqual(tree);
  });
});
