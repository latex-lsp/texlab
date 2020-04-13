use futures_boxed::boxed;
use image::{png::PNGEncoder, ColorType, DynamicImage, GenericImageView, ImageBuffer, RgbaImage};
use io::Cursor;
use log::warn;
use std::{io, process::Stdio, time::Duration};
use tempfile::TempDir;
use texlab_components::COMPONENT_DATABASE;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{
    ClientCapabilitiesExt, Hover, HoverContents, MarkupContent, MarkupKind, Range, RangeExt,
    TextDocumentPositionParams,
};
use texlab_syntax::{latex, CharStream, LatexIncludeKind, SyntaxNode};
use texlab_tex::{CompileError, CompileParams, DistributionKind, Format};
use thiserror::Error;
use tokio::process::Command;

const PREVIEW_ENVIRONMENTS: &[&str] = &[
    "align",
    "alignat",
    "aligned",
    "alignedat",
    "algorithmic",
    "array",
    "Bmatrix",
    "bmatrix",
    "cases",
    "CD",
    "eqnarray",
    "equation",
    "gather",
    "gathered",
    "matrix",
    "multline",
    "pmatrix",
    "smallmatrix",
    "split",
    "subarray",
    "Vmatrix",
    "vmatrix",
];

const IGNORED_PACKAGES: &[&str] = &["biblatex", "pgf", "tikz"];

#[derive(Debug, Clone, Copy)]
enum MathElement {
    Environment(latex::Environment),
    Equation(latex::Equation),
    Inline(latex::Inline),
}

impl MathElement {
    fn range(&self, tree: &latex::Tree) -> Range {
        match self {
            Self::Environment(env) => env.range(tree),
            Self::Equation(eq) => eq.range(tree),
            Self::Inline(inline) => inline.range(tree),
        }
    }
}

#[derive(Debug, Error)]
enum RenderError {
    #[error("an I/O error occurred: `{0}`")]
    IO(#[from] io::Error),
    #[error("a compile error occurred: `{0}`")]
    Compile(#[from] CompileError),
    #[error("compilation failed")]
    DviNotFound,
    #[error("dvipng is not installed")]
    DviPngNotInstalled,
    #[error("calling dvipng failed")]
    DviPngFaulty,
    #[error("failed to decode image")]
    DecodeImage,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexPreviewHoverProvider;

impl LatexPreviewHoverProvider {
    fn is_preview_environment(
        req: &FeatureRequest<TextDocumentPositionParams>,
        table: &latex::SymbolTable,
        environment: latex::Environment,
    ) -> bool {
        let canonical_name = environment
            .left
            .name(&table)
            .map(latex::Token::text)
            .unwrap_or_default()
            .replace('*', "");

        PREVIEW_ENVIRONMENTS.contains(&canonical_name.as_ref())
            || Self::theorem_environments(req).contains(&canonical_name.as_ref())
    }

    fn theorem_environments(req: &FeatureRequest<TextDocumentPositionParams>) -> Vec<&str> {
        let mut names = Vec::new();
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                table
                    .theorem_definitions
                    .iter()
                    .map(|thm| thm.name(&table).text())
                    .for_each(|thm| names.push(thm));
            }
        }
        names
    }

    async fn render(
        req: &FeatureRequest<TextDocumentPositionParams>,
        range: Range,
    ) -> Result<Hover, RenderError> {
        let code = Self::generate_code(req, range);
        let params = CompileParams {
            file_name: "preview.tex",
            code: &code,
            format: Format::Latex,
            timeout: Duration::from_secs(10),
        };
        let dir = req.distro.0.compile(params).await?.dir;
        if !dir.path().join("preview.dvi").exists() {
            return Err(RenderError::DviNotFound);
        }

        let img = Self::add_margin(Self::dvipng(&dir).await?);
        let base64 = Self::encode_image(img);
        let markdown = format!("![preview](data:image/png;base64,{})", base64);
        dir.close()?;
        Ok(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: Some(range),
        })
    }

    fn generate_code(req: &FeatureRequest<TextDocumentPositionParams>, range: Range) -> String {
        let mut code = String::new();
        code.push_str("\\documentclass{article}\n");
        code.push_str("\\thispagestyle{empty}\n");
        Self::generate_includes(req, &mut code);
        Self::generate_command_definitions(req, &mut code);
        Self::generate_math_operators(req, &mut code);
        Self::generate_theorem_definitions(req, &mut code);
        code.push_str("\\begin{document}\n");
        code.push_str(&CharStream::extract(&req.current().text, range));
        code.push('\n');
        code.push_str("\\end{document}\n");
        code
    }

    fn generate_includes(req: &FeatureRequest<TextDocumentPositionParams>, code: &mut String) {
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                let text = &doc.text;
                for include in &table.includes {
                    if include.kind == LatexIncludeKind::Package {
                        if include
                            .paths(&table)
                            .iter()
                            .all(|path| IGNORED_PACKAGES.contains(&path.text()))
                        {
                            continue;
                        }

                        if include
                            .paths(&table)
                            .iter()
                            .map(|path| format!("{}.sty", path.text()))
                            .any(|name| !COMPONENT_DATABASE.exists(&name))
                        {
                            continue;
                        }

                        code.push_str(&CharStream::extract(&text, table[include.parent].range()));
                        code.push('\n');
                    }
                }
            }
        }
    }

    fn generate_command_definitions(
        req: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                table
                    .command_definitions
                    .iter()
                    .map(|def| CharStream::extract(&doc.text, table[def.parent].range()))
                    .for_each(|def| {
                        code.push_str(&def);
                        code.push('\n');
                    });
            }
        }
    }

    fn generate_math_operators(
        req: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                table
                    .math_operators
                    .iter()
                    .map(|op| CharStream::extract(&doc.text, table[op.parent].range()))
                    .for_each(|op| {
                        code.push_str(&op);
                        code.push('\n');
                    });
            }
        }
    }

    fn generate_theorem_definitions(
        req: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                table
                    .theorem_definitions
                    .iter()
                    .map(|thm| CharStream::extract(&doc.text, table[thm.parent].range()))
                    .for_each(|thm| {
                        code.push_str(&thm);
                        code.push('\n');
                    })
            }
        }
    }

    async fn dvipng(dir: &TempDir) -> Result<DynamicImage, RenderError> {
        let process = Command::new("dvipng")
            .args(&["-D", "175", "-T", "tight", "preview.dvi"])
            .current_dir(dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| RenderError::DviPngNotInstalled)?;

        process.await.map_err(|_| RenderError::DviPngFaulty)?;

        let png_file = dir.path().join("preview1.png");
        let png = image::open(png_file).map_err(|_| RenderError::DecodeImage)?;
        Ok(png)
    }

    fn add_margin(image: DynamicImage) -> RgbaImage {
        let margin = 5;
        let width = image.width() + 2 * margin;
        let height = image.height() + 2 * margin;
        let mut result =
            ImageBuffer::from_pixel(width, height, image::Rgba([0xFF, 0xFF, 0xFF, 0xFF]));

        for x in 0..image.width() {
            for y in 0..image.height() {
                let pixel = image.get_pixel(x, y);
                result.put_pixel(x + margin, y + margin, pixel);
            }
        }
        result
    }

    fn encode_image(image: RgbaImage) -> String {
        let mut image_buf = Cursor::new(Vec::new());
        let png_encoder = PNGEncoder::new(&mut image_buf);
        let width = image.width();
        let height = image.height();
        png_encoder
            .encode(&image.into_raw(), width, height, ColorType::Rgba8)
            .unwrap();
        base64::encode(&image_buf.into_inner())
    }
}

impl FeatureProvider for LatexPreviewHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if !req.client_capabilities.has_hover_markdown_support()
            || req.distro.0.kind() == DistributionKind::Tectonic
        {
            return None;
        }

        if let DocumentContent::Latex(table) = &req.current().content {
            let mut elements = Vec::new();
            table
                .inlines
                .iter()
                .map(|inline| MathElement::Inline(*inline))
                .for_each(|inline| elements.push(inline));

            table
                .equations
                .iter()
                .map(|eq| MathElement::Equation(*eq))
                .for_each(|eq| elements.push(eq));

            table
                .environments
                .iter()
                .filter(|env| Self::is_preview_environment(req, table, **env))
                .map(|env| MathElement::Environment(*env))
                .for_each(|env| elements.push(env));

            let range = elements
                .iter()
                .map(|elem| elem.range(&table))
                .find(|range| range.contains(req.params.position))?;

            return match Self::render(req, range).await {
                Ok(hover) => Some(hover),
                Err(why) => {
                    warn!("Preview failed: {}", why);
                    None
                }
            };
        }
        None
    }
}
