import { Location, ReferenceParams } from 'vscode-languageserver';
import { FeatureProvider } from '../provider';

export type ReferenceProvider = FeatureProvider<ReferenceParams, Location[]>;
