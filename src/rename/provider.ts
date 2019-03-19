import { RenameParams, WorkspaceEdit } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type RenameProvider = FeatureProvider<
  RenameParams,
  WorkspaceEdit | undefined
>;
