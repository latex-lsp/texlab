import { Diagnostic, Range } from 'vscode-languageserver';
import { runSingleFile } from '../workspaceBuilder';
import {
  BibtexEntryDiagnosticsProvider,
  createDiagnostic,
  ErrorCode,
} from './bibtexEntry';

describe('BibtexEntryDiagnosticsProvider', () => {
  const provider = BibtexEntryDiagnosticsProvider;

  it('should raise an error if the opening brace is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingBeginBrace, Range.create(0, 8, 0, 8)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if the entry key is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingEntryName, Range.create(0, 9, 0, 9)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if the comma after entry name is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingComma, Range.create(0, 12, 0, 12)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if the closing brace is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingEndBrace, Range.create(0, 13, 0, 13)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should should raise an error if "=" is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar}',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingAssign, Range.create(0, 17, 0, 17)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if a field has no content', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,\nbar = }',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingContent, Range.create(1, 5, 1, 5)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if two fields are not separated by a comma', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo,\nfoo = bar\nbaz = qux}',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingComma, Range.create(1, 9, 1, 9)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if a quote is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar =\n"}',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingQuote, Range.create(1, 1, 1, 1)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if a closing brace is missing', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar =\n{',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingEndBrace, Range.create(1, 1, 1, 1)),
      createDiagnostic(ErrorCode.MissingEndBrace, Range.create(1, 1, 1, 1)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should raise an error if a concat operation has no right side', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = baz \n# }',
      line: 0,
      character: 0,
    });
    const expected: Diagnostic[] = [
      createDiagnostic(ErrorCode.MissingContent, Range.create(1, 1, 1, 1)),
    ];
    expect(actual).toEqual(expected);
  });

  it('should not raise an error if the entry is valid', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article{foo, bar = "baz {qux}" # quux}',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });

  it('should not process LaTeX documents', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '@article',
      line: 0,
      character: 0,
    });
    expect(actual).toEqual([]);
  });
});
