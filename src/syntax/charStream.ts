import { Position } from 'vscode-languageserver';

export class CharStream {
  public index: number;
  private line: number;
  private character: number;

  constructor(public text: string) {
    this.line = 0;
    this.character = 0;
    this.index = 0;
  }

  public get position(): Position {
    return {
      line: this.line,
      character: this.character,
    };
  }

  public get available(): boolean {
    return this.index < this.text.length;
  }

  public peek(lookAhead: number = 0): string {
    return this.text.charAt(this.index + lookAhead);
  }

  public next(): string {
    if (this.text[this.index] === '\n') {
      this.line++;
      this.character = 0;
    } else {
      this.character++;
    }
    return this.text[this.index++];
  }

  public seek(position: Position) {
    while (this.available && this.line < position.line) {
      this.next();
    }

    while (this.available && this.character < position.character) {
      this.next();
    }
  }

  public skipRestOfLine() {
    while (this.available) {
      if (this.peek() === '\n') {
        this.next();
        break;
      } else {
        this.next();
      }
    }
  }
}
