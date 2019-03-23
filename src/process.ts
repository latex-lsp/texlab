import cp from 'child_process';
import kill from 'tree-kill';
import { CancellationToken } from 'vscode-jsonrpc';

const { ELECTRON_RUN_AS_NODE, ...env } = process.env;

interface ProcessInfo {
  executable: string;
  args: string[];
  directory?: string;
  stdout?: DataHandler;
  stderr?: DataHandler;
  stdin?: string;
}

export type DataHandler = (data: Buffer | string) => void;

export enum ProcessStatus {
  Success = 0,
  Error = 1,
  Failure = 2,
}

export class ProcessBuilder {
  private info: ProcessInfo;

  constructor(executable: string) {
    this.info = { executable, args: [] };
  }

  public args(...args: string[]): ProcessBuilder {
    this.info.args = args;
    return this;
  }

  public output(handler: DataHandler): ProcessBuilder {
    this.info.stdout = handler;
    return this;
  }

  public error(handler: DataHandler): ProcessBuilder {
    this.info.stderr = handler;
    return this;
  }

  public input(text: string): ProcessBuilder {
    this.info.stdin = text;
    return this;
  }

  public directory(directory: string): ProcessBuilder {
    this.info.directory = directory;
    return this;
  }

  public async start(
    cancellationToken?: CancellationToken,
    keepAlive: boolean = false,
  ): Promise<ProcessStatus> {
    return new Promise((resolve, reject) => {
      const { executable, args, directory, stdout, stderr, stdin } = this.info;
      const process = cp.spawn(executable, args, {
        env,
        cwd: directory,
      });

      if (stdout) {
        process.stdout.on('data', stdout);
      }

      if (stderr) {
        process.stderr.on('data', stderr);
      }

      if (stdin) {
        process.stdin.write(stdin);
        process.stdin.end();
      }

      if (cancellationToken) {
        cancellationToken.onCancellationRequested(() => {
          if (!keepAlive) {
            kill(process.pid);
          }

          reject();
        });
      }

      process.on('exit', code => {
        const status = code === 0 ? ProcessStatus.Success : ProcessStatus.Error;
        resolve(status);
      });

      process.on('error', () => resolve(ProcessStatus.Failure));
    });
  }
}
