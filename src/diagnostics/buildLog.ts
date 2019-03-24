import { EOL } from 'os';
import path from 'path';
import { Uri } from '../uri';

export enum BuildErrorKind {
  Error,
  Warning,
}

export interface BuildError {
  uri: Uri;
  kind: BuildErrorKind;
  message: string;
  line?: number;
}

export function parseLog(parent: Uri, log: string): BuildError[] {
  const errors: BuildError[] = [];
  log = prepareLog(log);

  const ranges = parseFileRanges(parent, log);
  function resolveFile(index: number) {
    const range = ranges.find(x => x.contains(index));
    return range === undefined ? parent : range.uri || parent;
  }

  let match;
  const errorRegex = /^! (((.|\r|\n)*?)\r?\nl\.(\d+)|([^\r\n]*))/gm;
  while ((match = errorRegex.exec(log))) {
    const message = (match[2] || match[5]).split(/\r?\n/)[0].trim();
    const line =
      match[4] === undefined ? undefined : parseInt(match[4], 10) - 1;

    errors.push({
      uri: resolveFile(match.index),
      message,
      kind: BuildErrorKind.Error,
      line,
    });
  }

  const badBoxRegex = /((Ov|Und)erfull \\[hv]box[^\r\n]*lines? (\d+)[^\r\n]*)/g;
  while ((match = badBoxRegex.exec(log))) {
    const message = match[1];
    const line = parseInt(match[3], 10) - 1;
    errors.push({
      uri: resolveFile(match.index),
      message,
      kind: BuildErrorKind.Warning,
      line,
    });
  }

  const warningRegex = /(LaTeX|Package [a-zA-Z_\-]+) Warning: ([^\r\n]*)/g;
  while ((match = warningRegex.exec(log))) {
    const message = match[2];
    errors.push({
      uri: resolveFile(match.index),
      message,
      kind: BuildErrorKind.Warning,
      line: undefined,
    });
  }

  return errors;
}

function prepareLog(log: string): string {
  const MAX_LINE_LENGTH = 79;
  const oldLines = log.split(/\r?\n/);
  const newLines: string[] = [];
  let index = 0;
  while (index < oldLines.length) {
    const line = oldLines[index];
    const match = line.match(/^\([a-zA-Z_\-]+\)\s*(.*)$/);
    // Remove the package name from the following warnings:
    //
    // Package biblatex Warning: 'babel/polyglossia' detected but 'csquotes' missing.
    // (biblatex)                Loading 'csquotes' recommended.
    if (match !== null) {
      newLines[newLines.length - 1] += ' ' + match[1];
    } else if (line.endsWith('...')) {
      newLines.push(line.substring(0, line.length - 3));
      newLines[newLines.length - 1] += oldLines[index++];
    } else if (line.length === MAX_LINE_LENGTH) {
      newLines.push(line);
      newLines[newLines.length - 1] += oldLines[index++];
    } else {
      newLines.push(line);
    }
    index++;
  }
  return newLines.join(EOL);
}

class FileRange {
  public readonly length: number;

  constructor(
    public readonly uri: Uri | undefined,
    public readonly start: number,
    public readonly end: number,
  ) {
    this.length = end - start + 1;
  }

  public contains(index: number) {
    return index >= this.start && index <= this.end;
  }
}

function parseFileRanges(parent: Uri, log: string): FileRange[] {
  const ranges: FileRange[] = [];
  const regex = /\(([^\r\n()]+\.(tex|sty|cls))/g;
  let match;
  while ((match = regex.exec(log))) {
    let balance = 1;
    let end = match.index + 1;
    while (balance > 0 && end < log.length) {
      if (log[end] === '(') {
        balance++;
      } else if (log[end] === ')') {
        balance--;
      }
      end++;
    }

    const basePath = path.dirname(parent.fsPath);
    const fullPath = path.resolve(basePath, match[1]);
    const uri = fullPath.startsWith(basePath) ? Uri.file(fullPath) : undefined;
    ranges.push(new FileRange(uri, match.index, end));
  }
  ranges.sort((x, y) => x.length - y.length);
  return ranges;
}
