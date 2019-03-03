package texlab.hover

import org.eclipse.lsp4j.MarkupContent
import org.eclipse.lsp4j.MarkupKind
import org.scilab.forge.jlatexmath.ParseException
import org.scilab.forge.jlatexmath.TeXConstants
import org.scilab.forge.jlatexmath.TeXFormula
import java.awt.Color
import java.awt.Insets
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import java.util.*
import javax.imageio.ImageIO
import javax.swing.JLabel
import javax.swing.JOptionPane

object LatexFormulaRenderer {
    val ENVIRONMENTS = arrayOf("align", "align", "alignat", "aligned", "alignedat",
            "array", "Bmatrix", "bmatrix", "cases", "CD", "eqnarray", "equation", "equation",
            "gather", "gathered", "matrix", "multline", "pmatrix", "smallmatrix",
            "split", "subarray", "Vmatrix", "vmatrix")

    fun render(text: String): MarkupContent? {
        var code = text
        for (environment in ENVIRONMENTS) {
            code = code.replace("\\begin{$environment*}", "\\begin{$environment}")
            code = code.replace("\\end{$environment*}", "\\end{$environment}")
        }
        code = code.replace("\\begin{equation}", "\\[")
        code = code.replace("\\end{equation}", "\\]")

        try {
            val formula = TeXFormula(code)
            val icon = formula.TeXIconBuilder()
                    .setStyle(TeXConstants.STYLE_DISPLAY)
                    .setSize(20f)
                    .build()
                    .apply { insets = Insets(5, 5, 5, 5) }
            val image = BufferedImage(icon.iconWidth, icon.iconHeight, BufferedImage.TYPE_INT_ARGB)

            val graphics = image.createGraphics()
            graphics.color = Color.white
            graphics.fillRect(0, 0, icon.iconWidth, icon.iconHeight)
            val label = JLabel().apply { foreground = Color.black }
            icon.paintIcon(label, graphics, 0, 0)

            val stream = ByteArrayOutputStream()
            stream.use {
                ImageIO.write(image, "png", stream)
                val bytes = stream.toByteArray()
                val base64 = Base64.getEncoder().encodeToString(bytes)
                return MarkupContent().apply {
                    kind = MarkupKind.MARKDOWN
                    value = "![formula](data:image/png;base64,$base64)"
                }
            }
        } catch (e: ParseException) {
            JOptionPane.showMessageDialog(null, code + "\n" + e.toString())
            return null
        }
    }
}

