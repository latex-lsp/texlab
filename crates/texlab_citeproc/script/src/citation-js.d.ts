declare module '@citation-js/core' {
  export class Cite {
    constructor(text: string);

    public format(type: string, options: any): string;
  }
}
