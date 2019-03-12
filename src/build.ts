import * as path from 'path';
import { RemoteConsole } from 'vscode-languageserver';
import { ProcessBuilder, ProcessStatus } from './process';
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

function toBuildResult(status: ProcessStatus): BuildResult {
  switch (status) {
    case ProcessStatus.Success:
      return { status: BuildStatus.Success };
    case ProcessStatus.Error:
      return { status: BuildStatus.Error };
    case ProcessStatus.Failure:
      return { status: BuildStatus.Failure };
  }
}

export const BuildProvider: BuildProviderFactory = (console, listener) => ({
  execute: async (context, cancellationToken) => {
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

    const appendLog = (data: string | Buffer) => {
      console.log(data.toString());
    };

    try {
      return toBuildResult(
        await new ProcessBuilder(config.executable)
          .args(...config.args, name)
          .directory(directory)
          .output(appendLog)
          .error(appendLog)
          .start(cancellationToken),
      );
    } finally {
      listener.progress({ ...progress, done: true });
    }
  },
});
