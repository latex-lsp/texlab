import { TokenBuffer } from './token';

describe('TokenBuffer', () => {
  function createBuffer<T>(items: T[]): TokenBuffer<T> {
    items.reverse();
    return new TokenBuffer({
      next() {
        return items.pop();
      },
    });
  }

  it('should return the correct item when peeking', () => {
    const buffer = createBuffer([1, 2, 3]);
    expect(buffer.peek(0)).toEqual(1);
    expect(buffer.peek(1)).toEqual(2);
    expect(buffer.peek(2)).toEqual(3);
  });

  it('should return the correct item when advancing', () => {
    const buffer = createBuffer([1, 2, 3]);
    expect(buffer.available).toBeTruthy();
    expect(buffer.next()).toEqual(1);
    expect(buffer.next()).toEqual(2);
    expect(buffer.next()).toEqual(3);
    expect(buffer.available).toBeFalsy();
  });
});
