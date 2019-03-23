import parser from 'fast-xml-parser';
import mathjax from 'mathjax-node';

mathjax.config({
  MathJax: {
    tex2jax: {
      inlineMath: [['$', '$'], ['\\(', '\\)']],
      processEscapes: true,
    },
  },
  displayErrors: false,
});

interface Svg {
  style: string;
  width: string;
  height: string;
  viewBox: string;
}

interface SvgContainer {
  svg: Svg;
}

const xmlParserOptions: Partial<parser.X2jOptions> = {
  attributeNamePrefix: '',
  ignoreAttributes: false,
  parseAttributeValue: true,
};

const j2xParser = new parser.j2xParser(xmlParserOptions);

const SVG_BACKGROUND = 'white';
const SVG_SCALE = 1.25;
const SVG_PADDING_FACTOR = 0.05;

export async function renderMath(
  math: string,
  inline: boolean,
): Promise<string> {
  const result = await mathjax.typeset({
    math,
    speakText: false,
    svg: true,
    format: inline ? 'inline-TeX' : 'TeX',
  });

  const container: SvgContainer = parser.parse(result.svg!, xmlParserOptions);
  const { svg } = container;
  svg.style += `background-color: ${SVG_BACKGROUND}`;

  scale(svg, SVG_SCALE);
  addPadding(svg, SVG_PADDING_FACTOR);

  return j2xParser.parse(container);
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
