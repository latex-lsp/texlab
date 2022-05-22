import * as cp from 'child_process';
import * as fs from 'fs';
import * as os from 'os';
import { promisify } from 'util';
import * as vscode from 'vscode';
import {
  DidChangeConfigurationNotification,
  ServerOptions,
  State,
} from 'vscode-languageclient/node';
import {
  BuildStatus,
  ForwardSearchStatus,
  LatexLanguageClient,
} from './client';
import {
  BIBTEX_FILE,
  BIBTEX_UNTITLED,
  LATEX_FILE,
  LATEX_UNTITLED,
} from './selectors';
import { ExtensionState, Messages, StatusIcon } from './view';

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  const serverCommand = await findServer(context);
  if (serverCommand === undefined) {
    vscode.window.showErrorMessage(
      'No pre-built binaries available for your platform. ' +
        'Please install the server manually. ' +
        'For more information, see https://github.com/latex-lsp/texlab.',
    );

    return;
  }

  const serverConfig = vscode.workspace.getConfiguration('texlab.server');
  const serverOptions = getServerOptions(serverCommand, serverConfig);
  const icon = new StatusIcon();
  const client = new LatexLanguageClient(
    'texlab',
    serverOptions,
    {
      documentSelector: [
        LATEX_FILE,
        LATEX_UNTITLED,
        BIBTEX_FILE,
        BIBTEX_UNTITLED,
      ],
      outputChannelName: 'TexLab Language Server',
      uriConverters: {
        code2Protocol: (uri) => uri.toString(true),
        protocol2Code: (value) => vscode.Uri.parse(value),
      },
    },
    icon,
  );

  context.subscriptions.push(
    vscode.commands.registerTextEditorCommand('latex.build', (editor) =>
      build(editor, client),
    ),
    vscode.commands.registerTextEditorCommand('latex.forwardSearch', (editor) =>
      forwardSearch(editor, client),
    ),
    client.onDidChangeState(({ newState }) => {
      icon.update(
        newState === State.Running
          ? ExtensionState.Running
          : ExtensionState.Stopped,
      );
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('texlab')) {
        client.sendNotification(DidChangeConfigurationNotification.type, {
          settings: {},
        });
      }
    }),
    icon,
  );

  client.start();
}

async function findServer(
  context: vscode.ExtensionContext,
): Promise<string | undefined> {
  try {
    await promisify(cp.execFile)('texlab', ['--version']);
    return 'texlab';
  } catch {
    const serverName = os.platform() === 'win32' ? 'texlab.exe' : 'texlab';
    const serverPath = context.asAbsolutePath(`server/${serverName}`);
    return (await promisify(fs.exists)(serverPath)) ? serverPath : undefined;
  }
}

function getServerOptions(
  serverCommand: string,
  serverConfig: vscode.WorkspaceConfiguration,
): ServerOptions {
  const trace = serverConfig.get<boolean>('trace');
  const logFilePath = serverConfig.get<string | undefined>('logFile');
  const args = [];
  if (trace) {
    args.push('-vvvv');
  }
  if (logFilePath) {
    args.push('--log-file');
    args.push(logFilePath);
  }

  const { ELECTRON_RUN_AS_NODE, ...env } = process.env;
  return {
    run: {
      command: serverCommand,
      args,
      options: {
        env,
      },
    },
    debug: {
      command: serverCommand,
      args,
      options: {
        env: {
          ...env,
          RUST_BACKTRACE: '1',
        },
      },
    },
  };
}

async function build(
  { document }: vscode.TextEditor,
  client: LatexLanguageClient,
): Promise<void> {
  if (
    vscode.languages.match([LATEX_FILE, BIBTEX_FILE], document) <= 0 ||
    (document.isDirty && !(await document.save()))
  ) {
    return;
  }

  const { status } = await client.build(document);
  switch (status) {
    case BuildStatus.Success:
      break;
    case BuildStatus.Cancelled:
      break;
    case BuildStatus.Error:
      vscode.window.showErrorMessage(Messages.BUILD_ERROR);
      break;
    case BuildStatus.Failure:
      vscode.window.showErrorMessage(Messages.BUILD_FAILURE);
      break;
  }
}

async function forwardSearch(
  { document, selection }: vscode.TextEditor,
  client: LatexLanguageClient,
): Promise<void> {
  if (vscode.languages.match(LATEX_FILE, document) <= 0) {
    return;
  }

  const { status } = await client.forwardSearch(document, selection.start);
  switch (status) {
    case ForwardSearchStatus.Success:
      break;
    case ForwardSearchStatus.Error:
    case ForwardSearchStatus.Failure:
      vscode.window.showErrorMessage(Messages.SEARCH_FAILURE);
      break;
    case ForwardSearchStatus.Unconfigured:
      vscode.window.showInformationMessage(Messages.SEARCH_UNCONFIGURED);
      break;
  }
}
