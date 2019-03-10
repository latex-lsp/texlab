import * as cp from 'child_process';
import * as path from 'path';
import { CancellationToken } from 'vscode-jsonrpc';
import { RemoteConsole } from 'vscode-languageserver';
import { BuildResult, BuildStatus } from './protocol/build';
import { ProgressListener, ProgressParams } from './protocol/progress';
import { Uri } from './uri';

export interface BuildConfig {
  executable: string;
  args: string[];
  onSave: boolean;
}

export async function buildDocument(
  uri: Uri,
  config: BuildConfig,
  listener: ProgressListener,
  console: RemoteConsole,
  cancellationToken: CancellationToken,
): Promise<BuildResult> {
  const name = path.basename(uri.fsPath);
  const directory = path.dirname(uri.fsPath);
  const progress: ProgressParams = {
    id: 'build',
    title: 'Building',
    message: name,
  };

  listener.progress(progress);

  const process = cp.spawn(config.executable, [...config.args, name], {
    cwd: directory,
  });

  const appendLog = (data: string | Buffer) => {
    console.log(data.toString());
  };

  process.stdout.on('data', appendLog);
  process.stderr.on('data', appendLog);

  return new Promise<BuildResult>((resolve, reject) => {
    cancellationToken.onCancellationRequested(() => {
      process.kill();
      reject();
    });

    process.on('exit', exitCode => {
      resolve({
        status: exitCode === 0 ? BuildStatus.Success : BuildStatus.Error,
      });
    });

    process.on('error', () => resolve({ status: BuildStatus.Failure }));
  }).finally(() => {
    listener.progress({ ...progress, done: true });
  });
}
