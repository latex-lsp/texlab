import { Hover, TextDocumentPositionParams } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type HoverProvider = FeatureProvider<
  TextDocumentPositionParams,
  Hover | undefined
>;
