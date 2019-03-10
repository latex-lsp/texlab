// tslint:disable: import-blacklist
import URI from 'vscode-uri';

declare module 'vscode-uri' {
  export default interface URI {
    isFile(): boolean;
    equals(other: URI): boolean;
  }
}

URI.prototype.isFile = function(this: URI) {
  return this.scheme === 'file';
};

URI.prototype.equals = function(this: URI, other: URI) {
  return this.toString() === other.toString();
};

export { default as Uri } from 'vscode-uri';
