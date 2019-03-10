import * as cp from 'child_process';
import * as path from 'path';
import { CancellationToken } from 'vscode-jsonrpc';
import { RemoteConsole } from 'vscode-languageserver';
import { FeatureContext, LanguageFeature } from './feature';
import { BuildResult, BuildStatus } from './protocol/build';
import { ProgressListener, ProgressParams } from './protocol/progress';

export interface BuildConfig {
  executable: string;
  args: string[];
  onSave: boolean;
}

export class BuildFeature implements LanguageFeature<BuildConfig, BuildResult> {
  constructor(
    private console: RemoteConsole,
    private progressListener: ProgressListener,
  ) {}

  public execute(
    context: FeatureContext<BuildConfig>,
    cancellationToken?: CancellationToken,
  ): Promise<BuildResult> {
    const { params: config, uri, workspace } = context;
    const parent = workspace.findParent(uri)!;
    const name = path.basename(parent.uri.fsPath);
    const directory = path.dirname(parent.uri.fsPath);

    const progress: ProgressParams = {
      id: 'build',
      title: 'Building',
      message: name,
    };
    this.progressListener.progress(progress);

    const process = cp.spawn(config.executable, [...config.args, name], {
      cwd: directory,
    });

    const appendLog = (data: string | Buffer) => {
      this.console.log(data.toString());
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
      this.progressListener.progress({ ...progress, done: true });
    });
  }
}
