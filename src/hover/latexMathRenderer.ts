import parser from 'fast-xml-parser';
import { liteAdaptor } from 'mathjax3/mathjax3/adaptors/liteAdaptor';
import { HTMLDocument } from 'mathjax3/mathjax3/handlers/html/HTMLDocument';
import { HTMLMathItem } from 'mathjax3/mathjax3/handlers/html/HTMLMathItem';
import { TeX } from 'mathjax3/mathjax3/input/tex';
import { AllPackages } from 'mathjax3/mathjax3/input/tex/AllPackages';
import { SVG } from 'mathjax3/mathjax3/output/svg';

// Workaround for math tables, see issue #184.
// https://github.com/mathjax/mathjax-v3/issues/184
(global as any).top = true;

const inputJax = new TeX({ packages: AllPackages });
const outputJax = new SVG();
const adaptor = liteAdaptor();
const html = new HTMLDocument('', adaptor, {
  InputJax: inputJax,
  OutputJax: outputJax,
});

type MathItem = HTMLMathItem<any, any, any>;

interface Svg {
  style: string;
  width: string;
  height: string;
  viewBox: string;
}

interface SvgContainer {
  'mjx-container': {
    svg: Svg;
  };
}

const xmlParserOptions: Partial<parser.X2jOptions> = {
  attributeNamePrefix: '',
  ignoreAttributes: false,
  parseAttributeValue: true,
};

const j2xParser = new parser.j2xParser(xmlParserOptions);
const svgOptions = {
  scale: 1.25,
  padding: 0.05,
  background: 'white',
  emSize: 16,
  exSize: 8,
  containerWidth: 500,
  lineWidth: Number.MAX_VALUE,
};

export function renderMath(texCode: string, inline: boolean): string {
  const math: MathItem = new html.options.MathItem(texCode, inputJax, !inline);
  math.setMetrics(
    svgOptions.emSize,
    svgOptions.exSize,
    svgOptions.containerWidth,
    svgOptions.lineWidth,
    1,
  );

  math.compile(html);
  math.typeset(html);

  const { 'mjx-container': container }: SvgContainer = parser.parse(
    adaptor.outerHTML(math.typesetRoot),
    xmlParserOptions,
  );

  const { svg } = container;
  addStyleProperty(svg, 'background-color', svgOptions.background);
  scale(svg, svgOptions.scale);
  addPadding(svg, svgOptions.padding);

  return j2xParser.parse(container);
}

function addStyleProperty(svg: Svg, key: string, value: string) {
  svg.style = [...svg.style.split(';'), `${key}: ${value}`].join(';');
}

function scale(svg: Svg, factor: number) {
  const transform = (x: string) => {
    const value = Number.parseFloat(x.substring(0, x.length - 2));
    return `${value * factor}ex`;
  };

  svg.width = transform(svg.width);
  svg.height = transform(svg.height);
}

function addPadding(svg: Svg, factor: number) {
  let [minX, minY, width, height] = svg.viewBox
    .split(' ')
    .map(x => Number.parseFloat(x));

  const paddingX = factor * (width - minX);
  const paddingY = factor * (height - minY);

  minX -= paddingX;
  minY -= paddingY;
  width += 2 * paddingX;
  height += 2 * paddingY;

  svg.viewBox = `${minX} ${minY} ${width} ${height}`;
}
