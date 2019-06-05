import { Cite } from '@citation-js/core';
import '@citation-js/plugin-bibtex';
import '@citation-js/plugin-csl';
import fs from 'fs';

const code = fs.readFileSync('entry.bib').toString();
const cite = new Cite(code);
const html = cite.format('bibliography', {
  format: 'html',
  template: 'apa',
  lang: 'en-US',
});
console.log(html);
