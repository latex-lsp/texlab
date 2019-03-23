import fs from 'fs';
import path from 'path';
import { Uri } from '../uri';
import { BuildError, BuildErrorKind, parseLog } from './buildLog';

describe('Build Log Parser', () => {
  function getUri(name: string) {
    let file = path.join(__dirname, name);
    for (let i = 0; i < 26; i++) {
      const upperCase = String.fromCharCode('A'.charCodeAt(0) + i);
      const lowerCase = String.fromCharCode('a'.charCodeAt(0) + i);
      file = file.replace(upperCase + ':', lowerCase + ':');
    }
    return Uri.file(file);
  }

  const parent = getUri('parent.tex');
  const child = getUri('child.tex');

  async function run(name: string, expected: BuildError[]) {
    const file = path.join(__dirname, '..', '..', 'test', 'logs', name);
    const text = await fs.promises.readFile(file);

    const actual = parseLog(parent, text.toString());

    expect(actual).toHaveLength(expected.length);
    for (let i = 0; i < expected.length; i++) {
      expect(actual[i].uri.equals(expected[i].uri)).toBeTruthy();
      expect(actual[i].kind).toEqual(expected[i].kind);
      expect(actual[i].message).toEqual(expected[i].message);
      expect(actual[i].line).toEqual(expected[i].line);
    }
  }

  it('should parse bad boxes', async () => {
    const expected: BuildError[] = [
      {
        uri: parent,
        message:
          'Overfull \\hbox (200.00162pt too wide) in paragraph at lines 8--9',
        kind: BuildErrorKind.Warning,
        line: 7,
      },
      {
        uri: parent,
        message: 'Overfull \\vbox (3.19998pt too high) detected at line 23',
        kind: BuildErrorKind.Warning,
        line: 22,
      },
    ];
    await run('bad-box.txt', expected);
  });

  it('should parse citation warnings', async () => {
    const expected: BuildError[] = [
      {
        uri: parent,
        kind: BuildErrorKind.Warning,
        message: "Citation `foo' on page 1 undefined on input line 6.",
        line: undefined,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Warning,
        message: 'There were undefined references.',
        line: undefined,
      },
    ];
    await run('citation-warning.txt', expected);
  });

  it('should find errors in related documents', async () => {
    const expected: BuildError[] = [
      {
        uri: child,
        kind: BuildErrorKind.Error,
        message: 'Undefined control sequence.',
        line: 0,
      },
    ];
    await run('child-error.txt', expected);
  });

  it('should parse package errors', async () => {
    const expected: BuildError[] = [
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message:
          "Package babel Error: Unknown option `foo'. Either you misspelled it or " +
          'the language definition file foo.ldf was not found.',
        line: 392,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message:
          "Package babel Error: You haven't specified a language option.",
        line: 425,
      },
    ];
    await run('package-error.txt', expected);
  });

  it('should parse package warnings', async () => {
    const expected: BuildError[] = [
      {
        uri: parent,
        kind: BuildErrorKind.Warning,
        message:
          "'babel/polyglossia' detected but 'csquotes' missing. Loading 'csquotes' recommended.",
        line: undefined,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Warning,
        message: 'There were undefined references.',
        line: undefined,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Warning,
        message:
          'Please (re)run Biber on the file: parent and rerun LaTeX afterwards.',
        line: undefined,
      },
    ];
    await run('package-warning.txt', expected);
  });

  it('should parse TeX errors', async () => {
    const expected: BuildError[] = [
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: 'Undefined control sequence.',
        line: 6,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: 'Missing $ inserted.',
        line: 7,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: 'Undefined control sequence.',
        line: 8,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: 'Missing { inserted.',
        line: 9,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: 'Missing $ inserted.',
        line: 9,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: 'Missing } inserted.',
        line: 9,
      },
      {
        uri: parent,
        kind: BuildErrorKind.Error,
        message: '==> Fatal error occurred, no output PDF file produced!',
        line: undefined,
      },
    ];
    await run('tex-error.txt', expected);
  });
});
