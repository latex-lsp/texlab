import { Cite } from '@citation-js/core';
import '@citation-js/plugin-bibtex';
import '@citation-js/plugin-csl';

export default function(code) {
  const cite = new Cite(code);
  const html = cite.format('bibliography', {
    format: 'html',
    template: 'apa',
    lang: 'en-US',
  });
  return html;
}
