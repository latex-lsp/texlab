declare module 'citation-js' {
  class Cite {
    constructor(text: string);

    public format(type: string, options: any): string;
  }

  export = Cite;
}
