import * as program from 'commander';
import * as metadata from '../package.json';
import { LatexLanguageServer } from './latexLanguageServer';

program
  .description(metadata.description)
  .version(metadata.version)
  .option('--node-ipc')
  .option('--stdio')
  .option('--socket');

program.parse(process.argv);

if (!program.nodeIpc && !program.stdio && !program.socket) {
  program.help();
}

const server = new LatexLanguageServer();
server.listen();
