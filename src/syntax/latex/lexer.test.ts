import { LatexLexer, LatexToken, LatexTokenKind } from './lexer';

describe('LaTeX Lexer', () => {
  function verify(
    lexer: LatexLexer,
    line: number,
    character: number,
    text: string,
    kind: LatexTokenKind,
  ) {
    const expected = new LatexToken({ line, character }, text, kind);
    expect(lexer.next()).toEqual(expected);
  }

  it('should be able to tokenize words', () => {
    const lexer = new LatexLexer('foo bar baz');
    verify(lexer, 0, 0, 'foo', LatexTokenKind.Word);
    verify(lexer, 0, 4, 'bar', LatexTokenKind.Word);
    verify(lexer, 0, 8, 'baz', LatexTokenKind.Word);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize commands', () => {
    const lexer = new LatexLexer('\\foo\\bar@baz');
    verify(lexer, 0, 0, '\\foo', LatexTokenKind.Command);
    verify(lexer, 0, 4, '\\bar@baz', LatexTokenKind.Command);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize escape sequences', () => {
    const lexer = new LatexLexer('\\foo*\n\\%\\**');
    verify(lexer, 0, 0, '\\foo*', LatexTokenKind.Command);
    verify(lexer, 1, 0, '\\%', LatexTokenKind.Command);
    verify(lexer, 1, 2, '\\*', LatexTokenKind.Command);
    verify(lexer, 1, 4, '*', LatexTokenKind.Word);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize group delimiters', () => {
    const lexer = new LatexLexer('{}[]');
    verify(lexer, 0, 0, '{', LatexTokenKind.BeginGroup);
    verify(lexer, 0, 1, '}', LatexTokenKind.EndGroup);
    verify(lexer, 0, 2, '[', LatexTokenKind.BeginOptions);
    verify(lexer, 0, 3, ']', LatexTokenKind.EndOptions);
    expect(lexer.next()).toBeUndefined();
  });

  it('should ignore line comments', () => {
    const lexer = new LatexLexer(' %foo \nfoo');
    verify(lexer, 1, 0, 'foo', LatexTokenKind.Word);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize star commands', () => {
    const lexer = new LatexLexer('\\foo*');
    verify(lexer, 0, 0, '\\foo*', LatexTokenKind.Command);
    expect(lexer.next()).toBeUndefined();
  });
});
