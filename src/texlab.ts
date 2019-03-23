import program from 'commander';
import { LatexLanguageServer } from './latexLanguageServer';

program
  .description('LaTeX Language Server')
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
