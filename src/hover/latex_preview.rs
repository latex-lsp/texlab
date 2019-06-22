use crate::data::language::LatexIncludeKind;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::*;
use crate::syntax::text::{CharStream, SyntaxNode};
use crate::syntax::SyntaxTree;
use crate::tex;
use futures::compat::*;
use futures_boxed::boxed;
use image::png::PNGEncoder;
use image::{DynamicImage, GenericImage, GenericImageView};
use lsp_types::*;
use std::borrow::Cow;
use std::ffi::OsString;
use std::io;
use std::io::Cursor;
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tokio_process::CommandExt;

const PREVIEW_ENVIRONMENTS: &[&str] = &[
    "align",
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

#[derive(Debug, PartialEq, Eq, Clone)]
enum MathElement<'a> {
    Environment(&'a LatexEnvironment),
    Equation(&'a LatexEquation),
    Inline(&'a LatexInline),
}

impl<'a> SyntaxNode for MathElement<'a> {
    fn range(&self) -> Range {
        match self {
            MathElement::Environment(environment) => environment.range(),
            MathElement::Equation(equation) => equation.range(),
            MathElement::Inline(inline) => inline.range(),
        }
    }
}

#[derive(Debug)]
enum RenderError {
    IO(io::Error),
    DviPngNotInstalled,
    DviPngFaulty,
    CannotDecodeImage,
}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        RenderError::IO(error)
    }
}

pub struct LatexPreviewHoverProvider;

impl LatexPreviewHoverProvider {
    fn is_math_environment(environment: &LatexEnvironment) -> bool {
        let canonical_name = environment
            .left
            .name()
            .map(LatexToken::text)
            .unwrap_or_default()
            .replace('*', "");
        PREVIEW_ENVIRONMENTS.contains(&canonical_name.as_ref())
    }

    async fn render(
        request: &FeatureRequest<TextDocumentPositionParams>,
        range: Range,
    ) -> Result<Hover, RenderError> {
        let code = Self::generate_code(request, range);
        let directory = tex::compile("preview.tex", &code, tex::Format::Latex)
            .await?
            .directory;
        if !directory.path().join("preview.dvi").exists() {
            return Err(RenderError::IO(io::ErrorKind::NotFound.into()));
        }

        let image = Self::dvipng(&directory).await?;
        let image = Self::add_margin(image);
        let base64 = Self::encode_image(image);
        let markdown = format!("![preview](data:image/png;base64,{})", base64);
        directory.close()?;
        Ok(Hover {
            range: Some(range),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: Cow::from(markdown),
            }),
        })
    }

    fn generate_code(request: &FeatureRequest<TextDocumentPositionParams>, range: Range) -> String {
        let mut code = String::new();
        code.push_str("\\documentclass{article}\n");
        code.push_str("\\thispagestyle{empty}\n");
        Self::generate_includes(request, &mut code);
        Self::generate_command_definitions(request, &mut code);
        Self::generate_math_operators(request, &mut code);
        code.push_str("\\begin{document}\n");
        code.push_str(&Self::extract_text(&request.document().text, range));
        code.push('\n');
        code.push_str("\\end{document}\n");
        code
    }

    fn generate_includes(request: &FeatureRequest<TextDocumentPositionParams>, code: &mut String) {
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let text = &request.document().text;
                tree.includes
                    .iter()
                    .filter(|include| include.kind == LatexIncludeKind::Package)
                    .filter(|include| !IGNORED_PACKAGES.contains(&include.path().text()))
                    .filter(|include| {
                        let name = format!("{}.sty", include.path().text());
                        request
                            .resolver
                            .files_by_name
                            .contains_key(&OsString::from(name))
                    })
                    .for_each(|include| {
                        code.push_str(&Self::extract_text(&text, include.command.range));
                        code.push('\n');
                    });
            }
        }
    }

    fn generate_command_definitions(
        request: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                tree.command_definitions
                    .iter()
                    .map(|def| Self::extract_text(&document.text, def.range()))
                    .for_each(|def| {
                        code.push_str(&def);
                        code.push('\n');
                    });
            }
        }
    }

    fn generate_math_operators(
        request: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                tree.math_operators
                    .iter()
                    .map(|op| Self::extract_text(&document.text, op.range()))
                    .for_each(|op| {
                        code.push_str(&op);
                        code.push('\n');
                    });
            }
        }
    }

    async fn dvipng(directory: &TempDir) -> Result<DynamicImage, RenderError> {
        let process = Command::new("dvipng")
            .args(&["-D", "175", "-T", "tight", "preview.dvi"])
            .current_dir(directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn_async()
            .map_err(|_| RenderError::DviPngNotInstalled)?;

        process
            .compat()
            .await
            .map_err(|_| RenderError::DviPngFaulty)?;

        let png_file = directory.path().join("preview1.png");
        let png = image::open(png_file).map_err(|_| RenderError::CannotDecodeImage)?;
        Ok(png)
    }

    fn add_margin(image: DynamicImage) -> DynamicImage {
        let margin = 5;
        let width = image.width() + 2 * margin;
        let height = image.height() + 2 * margin;
        let mut result = DynamicImage::new_rgb8(width, height);
        for x in 0..result.width() {
            for y in 0..result.height() {
                result.put_pixel(x, y, image::Rgba([0xFF, 0xFF, 0xFF, 0xFF]))
            }
        }

        for x in 0..image.width() {
            for y in 0..image.height() {
                let pixel = image.get_pixel(x, y);
                result.put_pixel(x + margin, y + margin, pixel);
            }
        }
        result
    }

    fn encode_image(image: DynamicImage) -> String {
        let mut image_buf = Cursor::new(Vec::new());
        let png_encoder = PNGEncoder::new(&mut image_buf);
        png_encoder
            .encode(
                &image.raw_pixels(),
                image.width(),
                image.height(),
                image.color(),
            )
            .unwrap();
        base64::encode(&image_buf.into_inner())
    }

    fn extract_text(text: &str, range: Range) -> String {
        let mut stream = CharStream::new(text);
        stream.seek(range.start);
        stream.start_span();
        stream.seek(range.end);
        stream.end_span().text
    }
}

impl FeatureProvider for LatexPreviewHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let mut elements = Vec::new();
            tree.environments
                .iter()
                .filter(|env| Self::is_math_environment(env))
                .map(MathElement::Environment)
                .for_each(|env| elements.push(env));

            tree.equations
                .iter()
                .map(MathElement::Equation)
                .for_each(|eq| elements.push(eq));

            tree.inlines
                .iter()
                .map(MathElement::Inline)
                .for_each(|inline| elements.push(inline));

            let range = elements
                .iter()
                .find(|elem| elem.range().contains(request.params.position))
                .map(MathElement::range)?;

            return Some(Self::render(request, range).await.ok()?);
        }
        None
    }
}
