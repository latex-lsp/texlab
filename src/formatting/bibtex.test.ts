import { EOL } from 'os';
import { BibtexSyntaxTree } from '../syntax/bibtex/analysis';
import { BibtexDeclarationSyntax } from '../syntax/bibtex/ast';
import { BibtexFormatter } from './bibtex';

describe('BibTeX Formatter', () => {
  function verify(
    source: string,
    expected: string,
    lineLength: number = 30,
    insertSpaces: boolean = true,
  ) {
    const tree = new BibtexSyntaxTree(source);
    const entry = tree.root.children[0] as BibtexDeclarationSyntax;
    const formatter = new BibtexFormatter(insertSpaces, 4, lineLength);
    const actual = formatter.format(entry);
    expect(actual).toEqual(expected);
  }

  it('should wrap long lines', () => {
    const source = [
      '@article{foo,',
      'bar = {Lorem ipsum dolor sit amet,',
      'consectetur adipiscing elit.}, }',
    ].join(EOL);
    const expected = [
      '@article{foo,',
      '    bar = {Lorem ipsum dolor',
      '           sit amet,',
      '           consectetur',
      '           adipiscing elit.},',
      '}',
    ].join(EOL);
    verify(source, expected);
  });

  it('should not wrap long lines with a line length of zero', () => {
    const source = [
      '@article{foo,',
      'bar = {Lorem ipsum dolor sit amet,',
      'consectetur adipiscing elit.}, }',
    ].join(EOL);
    const expected = [
      '@article{foo,',
      '    bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.},',
      '}',
    ].join(EOL);
    verify(source, expected, 0);
  });

  it('should insert trailing commas', () => {
    const source = '@article{foo, bar = baz}';
    const expected = `@article{foo,${EOL}    bar = baz,${EOL}}`;
    verify(source, expected);
  });

  it('should insert missing braces', () => {
    const source = '@article{foo, bar = baz,';
    const expected = `@article{foo,${EOL}    bar = baz,${EOL}}`;
    verify(source, expected);
  });

  it('should handle commands', () => {
    const source = `@article{foo, bar = "\\baz",}`;
    const expected = `@article{foo,${EOL}    bar = "\\baz",${EOL}}`;
    verify(source, expected);
  });

  it('should handle string concatenation', () => {
    const source = `@article{foo, bar = "baz" # "qux"}`;
    const expected = `@article{foo,${EOL}    bar = "baz" # "qux",${EOL}}`;
    verify(source, expected);
  });

  it('should replace parens with braces', () => {
    const source = '@article(foo,)';
    const expected = `@article{foo,${EOL}}`;
    verify(source, expected);
  });

  it('should handle valid strings', () => {
    const source = '@string{foo="bar"}';
    const expected = `@string{foo = "bar"}`;
    verify(source, expected);
  });

  it('should handle invalid strings', () => {
    verify('@string{', '@string{');
    verify('@string{foo = ', '@string{foo = ');
  });

  it('should handle valid preambles', () => {
    const source = '@preamble{\n"foo bar baz"}';
    const expected = '@preamble{"foo bar baz"}';
    verify(source, expected);
  });

  it('should handle invalid preambles', () => {
    verify('@preamble{', '@preamble{');
  });

  it('should handle invalid entries', () => {
    verify('@entry{', '@entry{');
    verify('@entry{foo, bar', `@entry{foo,${EOL}    bar = }`);
  });

  it('should handle tabs', () => {
    const source = '@article{foo, bar = baz}';
    const expected = `@article{foo,${EOL}\tbar = baz,${EOL}}`;
    verify(source, expected, 30, false);
  });

  it('should handle invalid content', () => {
    const source = '@article{foo, bar = baz #}';
    const expected = `@article{foo,${EOL}    bar = baz #,${EOL}}`;
    verify(source, expected);
  });

  it('should handle unclosed braces', () => {
    const source = '@article{foo, bar = {';
    const expected = `@article{foo,${EOL}    bar = {,${EOL}}`;
    verify(source, expected);
  });
});
