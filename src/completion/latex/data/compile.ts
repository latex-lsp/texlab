import { randomBytes } from 'crypto';
import del from 'del';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { CancellationTokenSource } from 'vscode-jsonrpc';
import { ProcessBuilder } from '../../../process';

export enum TexFormat {
  Latex,
  Lualatex,
  Xelatex,
}

const TIMEOUT = 10000;

export async function compile(
  code: string,
  format: TexFormat,
): Promise<string | undefined> {
  return withTempDirectory(async directory => {
    const file = path.join(directory, 'code.tex');
    await fs.promises.writeFile(file, code);

    let executable: string;
    switch (format) {
      case TexFormat.Lualatex:
        executable = 'lualatex';
        break;
      case TexFormat.Xelatex:
        executable = 'xelatex';
        break;
      case TexFormat.Latex:
      default:
        executable = 'latex';
        break;
    }

    const cancellationTokenSource = new CancellationTokenSource();
    setTimeout(() => cancellationTokenSource.cancel(), TIMEOUT);

    try {
      await new ProcessBuilder(executable)
        .args('-interaction=batchmode', '-shell-escape', 'code.tex')
        .directory(directory)
        .start(cancellationTokenSource.token);
    } catch {
      return undefined;
    }

    const logFile = path.join(directory, 'code.log');
    if (fs.existsSync(logFile)) {
      return fs.promises.readFile(logFile).then(x => x.toString());
    }

    return undefined;
  });
}

async function withTempDirectory<T>(action: (directory: string) => Promise<T>) {
  const name = randomBytes(12).toString('hex');
  const directory = path.join(os.tmpdir(), name);
  await fs.promises.mkdir(directory);

  try {
    return await action(directory);
  } finally {
    await del(directory, { force: true });
  }
}
