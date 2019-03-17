import * as program from 'commander';
import * as metadata from '../package.json';
import { LatexLanguageServer } from './latexLanguageServer';

program
  .description(metadata.description)
  .version(metadata.version)
  .option('--node-ipc')
  .option('--stdio')
  .option('--socket')
  .option('--clientProcessId');

program.parse(process.argv);

try {
  const server = new LatexLanguageServer();
  server.listen();
} catch {
  program.help();
}
