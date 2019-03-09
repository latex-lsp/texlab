import { CharStream } from './charStream';

describe('CharStream', () => {
  it('should update the position when seeking', () => {
    const stream = new CharStream('foo\nbar-baz');
    const position = { line: 1, character: 2 };
    stream.seek(position);
    expect(stream.position).toEqual(position);
  });

  it('should update the position when advancing', () => {
    const stream = new CharStream('a\nb');
    stream.next();
    expect(stream.position).toEqual({ line: 0, character: 1 });
    stream.next();
    expect(stream.position).toEqual({ line: 1, character: 0 });
    stream.next();
    expect(stream.position).toEqual({ line: 1, character: 1 });
  });

  it('should not change the position when peeking', () => {
    const stream = new CharStream('a\nb');
    stream.peek();
    expect(stream.position).toEqual({ line: 0, character: 0 });
  });

  it('should return the correct character when peeking', () => {
    const stream = new CharStream('abc');
    expect(stream.peek(0)).toEqual('a');
    expect(stream.peek(1)).toEqual('b');
    expect(stream.peek(2)).toEqual('c');
  });

  it('should be able to skip the rest of the line', () => {
    const stream = new CharStream('foo\nbar');
    stream.skipRestOfLine();
    expect(stream.position).toEqual({ line: 1, character: 0 });
  });
});
