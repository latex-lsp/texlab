import { Location, TextDocumentPositionParams } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type DefinitionProvider = FeatureProvider<
  TextDocumentPositionParams,
  Location[]
>;
