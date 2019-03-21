declare module 'mathjax-node' {
  export type ConfigOptions = Partial<{
    displayMessages: boolean;
    displayErrors: boolean;
    undefinedCharError: boolean;
    extensions: string;
    fontURL: string;
    paths: any;
    MathJax: any;
  }>;

  export type TypesetOptions = Partial<{
    ex: number;
    width: number;
    useFontCache: boolean;
    useGlobalCache: boolean;
    linebreaks: boolean;
    equationNumbers: 'none' | 'AMS' | 'all';
    cjkCharWidth: number;

    math: string;
    format: 'TeX' | 'inline-TeX' | 'AsciiMath' | 'MathML';
    xmlns: string;

    html: boolean;
    htmlNode: boolean;
    css: boolean;
    mml: boolean;
    mmlNode: boolean;
    svg: boolean;
    svgNode: boolean;

    speakText: boolean;

    state: any;
    timeout: number;
  }>;

  export type TypesetResult = Partial<{
    errors: any[];
    mml: string;
    mmlNode: any;
    html: string;
    htmlNode: any;
    css: string;
    svg: string;
    svgNode: any;
    style: string;
    height: string;
    width: string;
    speakText: string;

    state: {
      glyphs: any[];
      defs: string;
      AMS: {
        startNumber: number;
        labels: string[];
        IDs: any[];
      };
    };
  }>;

  export function config(options: ConfigOptions): void;

  export function start(): void;

  export function typeset(options: TypesetOptions): Promise<TypesetResult>;
}
