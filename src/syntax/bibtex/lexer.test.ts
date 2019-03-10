import { BibtexLexer, BibtexToken, BibtexTokenKind } from './lexer';

describe('BibTeX Lexer', () => {
  function verify(
    lexer: BibtexLexer,
    line: number,
    character: number,
    text: string,
    kind: BibtexTokenKind,
  ) {
    const expected = new BibtexToken({ line, character }, text, kind);
    expect(lexer.next()).toEqual(expected);
  }

  it('should be able to tokenize words', () => {
    const lexer = new BibtexLexer('foo bar baz');
    verify(lexer, 0, 0, 'foo', BibtexTokenKind.Word);
    verify(lexer, 0, 4, 'bar', BibtexTokenKind.Word);
    verify(lexer, 0, 8, 'baz', BibtexTokenKind.Word);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize commands', () => {
    const lexer = new BibtexLexer('\\foo\\bar@baz');
    verify(lexer, 0, 0, '\\foo', BibtexTokenKind.Command);
    verify(lexer, 0, 4, '\\bar@baz', BibtexTokenKind.Command);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize escape sequences', () => {
    const lexer = new BibtexLexer('\\foo*\n\\%\\**');
    verify(lexer, 0, 0, '\\foo*', BibtexTokenKind.Command);
    verify(lexer, 1, 0, '\\%', BibtexTokenKind.Command);
    verify(lexer, 1, 2, '\\*', BibtexTokenKind.Command);
    verify(lexer, 1, 4, '*', BibtexTokenKind.Word);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize delimiters', () => {
    const lexer = new BibtexLexer('{}()"');
    verify(lexer, 0, 0, '{', BibtexTokenKind.BeginBrace);
    verify(lexer, 0, 1, '}', BibtexTokenKind.EndBrace);
    verify(lexer, 0, 2, '(', BibtexTokenKind.BeginParen);
    verify(lexer, 0, 3, ')', BibtexTokenKind.EndParen);
    verify(lexer, 0, 4, '"', BibtexTokenKind.Quote);
    expect(lexer.next()).toBeUndefined();
  });

  it('should be able to tokenize types', () => {
    const lexer = new BibtexLexer('@pReAmBlE\n@article\n@string');
    verify(lexer, 0, 0, '@pReAmBlE', BibtexTokenKind.PreambleType);
    verify(lexer, 1, 0, '@article', BibtexTokenKind.EntryType);
    verify(lexer, 2, 0, '@string', BibtexTokenKind.StringType);
  });

  it('should be able to tokenize operators', () => {
    const lexer = new BibtexLexer('=,#');
    verify(lexer, 0, 0, '=', BibtexTokenKind.Assign);
    verify(lexer, 0, 1, ',', BibtexTokenKind.Comma);
    verify(lexer, 0, 2, '#', BibtexTokenKind.Concat);
  });
});
