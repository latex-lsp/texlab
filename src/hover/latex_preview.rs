use crate::capabilities::ClientCapabilitiesExt;
use crate::completion::DATABASE;
use crate::range::RangeExt;
use crate::syntax::*;
use crate::tex;
use crate::workspace::*;
use futures_boxed::boxed;
use image::png::PNGEncoder;
use image::{DynamicImage, GenericImage, GenericImageView};
use log::*;
use lsp_types::*;
use std::io;
use std::io::Cursor;
use std::process::Stdio;
use tempfile::TempDir;
use tokio_net::process::Command;

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
    Compile(tex::CompileError),
    DviNotFound,
    DviPngNotInstalled,
    DviPngFaulty,
    DecodeImage,
}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        RenderError::IO(error)
    }
}

impl From<tex::CompileError> for RenderError {
    fn from(error: tex::CompileError) -> Self {
        RenderError::Compile(error)
    }
}

pub struct LatexPreviewHoverProvider;

impl LatexPreviewHoverProvider {
    fn is_math_environment(
        request: &FeatureRequest<TextDocumentPositionParams>,
        environment: &LatexEnvironment,
    ) -> bool {
        let canonical_name = environment
            .left
            .name()
            .map(LatexToken::text)
            .unwrap_or_default()
            .replace('*', "");

        PREVIEW_ENVIRONMENTS.contains(&canonical_name.as_ref())
            || Self::theorem_environments(request).contains(&canonical_name.as_ref())
    }

    fn theorem_environments(request: &FeatureRequest<TextDocumentPositionParams>) -> Vec<&str> {
        let mut names = Vec::new();
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                tree.math
                    .theorem_definitions
                    .iter()
                    .map(|thm| thm.name().text())
                    .for_each(|thm| names.push(thm));
            }
        }
        names
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
            return Err(RenderError::DviNotFound);
        }

        let image = Self::add_margin(Self::dvipng(&directory).await?);
        let base64 = Self::encode_image(image);
        let markdown = format!("![preview](data:image/png;base64,{})", base64);
        directory.close()?;
        Ok(Hover {
            range: Some(range),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
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
        Self::generate_theorem_definitions(request, &mut code);
        code.push_str("\\begin{document}\n");
        code.push_str(&CharStream::extract(&request.document().text, range));
        code.push('\n');
        code.push_str("\\end{document}\n");
        code
    }

    fn generate_includes(request: &FeatureRequest<TextDocumentPositionParams>, code: &mut String) {
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let text = &request.document().text;
                for include in &tree.includes {
                    if include.kind == LatexIncludeKind::Package {
                        if include
                            .paths()
                            .iter()
                            .all(|path| IGNORED_PACKAGES.contains(&path.text()))
                        {
                            continue;
                        }

                        if include
                            .paths()
                            .iter()
                            .map(|path| format!("{}.sty", path.text()))
                            .any(|name| !DATABASE.exists(&name))
                        {
                            continue;
                        }

                        code.push_str(&CharStream::extract(&text, include.command.range));
                        code.push('\n');
                    }
                }
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
                    .map(|def| CharStream::extract(&document.text, def.range()))
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
                tree.math
                    .operators
                    .iter()
                    .map(|op| CharStream::extract(&document.text, op.range()))
                    .for_each(|op| {
                        code.push_str(&op);
                        code.push('\n');
                    });
            }
        }
    }

    fn generate_theorem_definitions(
        request: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for document in request.related_documents() {
            if let SyntaxTree::Latex(tree) = &document.tree {
                tree.math
                    .theorem_definitions
                    .iter()
                    .map(|thm| CharStream::extract(&document.text, thm.range()))
                    .for_each(|thm| {
                        code.push_str(&thm);
                        code.push('\n');
                    })
            }
        }
    }

    async fn dvipng(directory: &TempDir) -> Result<DynamicImage, RenderError> {
        let process = Command::new("dvipng")
            .args(&["-D", "175", "-T", "tight", "preview.dvi"])
            .current_dir(directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| RenderError::DviPngNotInstalled)?;

        process.await.map_err(|_| RenderError::DviPngFaulty)?;

        let png_file = directory.path().join("preview1.png");
        let png = image::open(png_file).map_err(|_| RenderError::DecodeImage)?;
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
}

impl FeatureProvider for LatexPreviewHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if !request.client_capabilities.has_hover_markdown_support() {
            return None;
        }

        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let mut elements = Vec::new();
            tree.math
                .inlines
                .iter()
                .map(MathElement::Inline)
                .for_each(|inline| elements.push(inline));

            tree.math
                .equations
                .iter()
                .map(MathElement::Equation)
                .for_each(|eq| elements.push(eq));

            tree.env
                .environments
                .iter()
                .filter(|env| Self::is_math_environment(request, env))
                .map(MathElement::Environment)
                .for_each(|env| elements.push(env));

            let range = elements
                .iter()
                .find(|elem| elem.range().contains(request.params.position))
                .map(MathElement::range)?;

            return match Self::render(request, range).await {
                Ok(hover) => Some(hover),
                Err(why) => {
                    let message = match why {
                        RenderError::IO(why) => format!("I/O error: {}", why),
                        RenderError::Compile(why) => match why {
                            tex::CompileError::Initialization => {
                                "compilation initialization failed".to_owned()
                            }
                            tex::CompileError::LatexNotInstalled => {
                                "latex not installed".to_owned()
                            }
                            tex::CompileError::Timeout => "compilation timed out".to_owned(),
                            tex::CompileError::Wait => "failed to wait for latex".to_owned(),
                            tex::CompileError::ReadLog => "failed to read log file".to_owned(),
                            tex::CompileError::Cleanup => "failed to cleanup latex".to_owned(),
                        },
                        RenderError::DviNotFound => "compilation failed".to_owned(),
                        RenderError::DviPngNotInstalled => "dvipng is not installed".to_owned(),
                        RenderError::DviPngFaulty => "dvipng failed".to_owned(),
                        RenderError::DecodeImage => "failed to decode image".to_owned(),
                    };
                    warn!("Preview failed: {}", message);
                    None
                }
            };
        }
        None
    }
}
