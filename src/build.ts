import * as cp from 'child_process';
import * as path from 'path';
import { RemoteConsole } from 'vscode-languageserver';
import { BuildResult, BuildStatus } from './protocol/build';
import { ProgressListener, ProgressParams } from './protocol/progress';
import { FeatureProvider } from './provider';

export interface BuildConfig {
  executable: string;
  args: string[];
  onSave: boolean;
}

export type BuildProvider = FeatureProvider<BuildConfig, BuildResult>;

type BuildProviderFactory = (
  console: RemoteConsole,
  listener: ProgressListener,
) => BuildProvider;

export const BuildProvider: BuildProviderFactory = (console, listener) => ({
  execute: (context, cancellationToken) => {
    const { params: config, uri, workspace } = context;
    const parent = workspace.findParent(uri)!;
    const name = path.basename(parent.uri.fsPath);
    const directory = path.dirname(parent.uri.fsPath);

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
      if (cancellationToken) {
        cancellationToken.onCancellationRequested(() => {
          process.kill();
          reject();
        });
      }

      process.on('exit', exitCode => {
        resolve({
          status: exitCode === 0 ? BuildStatus.Success : BuildStatus.Error,
        });
      });

      process.on('error', () => resolve({ status: BuildStatus.Failure }));
    }).finally(() => {
      listener.progress({ ...progress, done: true });
    });
  },
});
