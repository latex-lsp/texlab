import { parse } from './parser';
import { print } from './printer';

describe('BibTeX Parser', () => {
  function verify(text: string) {
    const tree1 = parse(text);
    const tree2 = parse(print(tree1));
    expect(tree2).toEqual(tree1);
  }

  it('should parse the empty document', () => {
    verify('');
  });

  it('should parse comments', () => {
    verify(`foo bar baz`);
  });

  it('should parse preambles', () => {
    verify(`@preamble`);
    verify(`@preamble{`);
    verify(`@preamble{ "`);
    verify(`@preamble{ "foo`);
    verify(`@preamble{ "foo" }`);
  });

  it('should parse strings', () => {
    verify(`@string`);
    verify(`@string{`);
    verify(`@string{cup`);
    verify(`@string{cup = `);
    verify(`@string{cup = "Cambridge University Press"`);
    verify(`@string{cup = "Cambridge University Press"}`);
  });

  it('should parse entries', () => {
    verify(`@article`);
    verify(`@article{`);
    verify(`@article{foo`);
    verify(`@article{foo,`);
    verify(`@article{foo, bar`);
    verify(`@article{foo, bar = `);
    verify(`@article{foo, bar = }`);
    verify(`@article{foo, bar = {baz}})`);
  });

  it('should parse content', () => {
    verify(`@article{foo, bar = {@baz @preamble @string}}`);
    verify(`@article{foo, bar = baz # }`);
    verify(`@article{foo, bar = baz # qux}`);
    verify(`@article{foo, bar = { \\baz }}`);
    verify(`@article{foo, bar = #}`);
    verify(`@article{foo, bar = (, baz = )}`);
    verify(`@article{foo,\nbar=baz}`);
  });
});
