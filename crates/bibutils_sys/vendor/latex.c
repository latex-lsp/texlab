/* 
 * latex.c
 *
 * convert between latex special chars and unicode
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "latex.h"

#define LATEX_COMBO (0)  /* 'combo' no need for protection on output */
#define LATEX_MACRO (1)  /* 'macro_name' to be protected by {\macro_name} on output */
#define LATEX_MATH  (2)  /* 'math_expression' to be protected by $math_expression$ on output */

struct latex_chars {
	unsigned int unicode;      /* unicode code point */
	unsigned char type;        /* LATEX_COMBO/LATEX_MACRO/LATEX_MATH */
	char *out;                 /* unadorned latex combination for output */
	char *variant[3];         /* possible variations on input */
};

static struct latex_chars latex_chars[] = { 

                                            /* LaTeX special characters */
   {  32, LATEX_COMBO, " ",              { "\\ ",                NULL,                 NULL               } }, /* escaping the space is used to avoid extra space after periods */
   {  35, LATEX_MACRO, "#",              { "\\#",                NULL,                 NULL               } }, /* Number/pound/hash sign */
   {  36, LATEX_MACRO, "$",              { "\\$",                NULL,                 NULL               } }, /* Dollar Sign */
   {  37, LATEX_MACRO, "%",              { "\\%",                NULL,                 NULL               } }, /* Percent Sign */
   {  38, LATEX_MACRO, "&",              { "\\&",                NULL,                 NULL               } }, /* Ampersand */
   {  95, LATEX_MACRO, "_",              { "\\_",                NULL,                 NULL               } }, /* Underscore alone indicates subscript */
   { 123, LATEX_MACRO, "{",              { "\\{",                "{\\textbraceleft}",  "\\textbraceleft"  } }, /* Left Curly Bracket */
   { 125, LATEX_MACRO, "}",              { "\\}",                "{\\textbraceright}", "\\textbraceright" } }, /* Right Curly Bracket */
   {  92, LATEX_MACRO, "backslash",      { "{\\backslash}",      "\\backslash",        NULL               } }, /* Backslash */
   { 176, LATEX_MACRO, "textdegree",     { "{\\textdegree}",     "\\textdegree",       "^\\circ"          } }, /* Degree sign */
   { 126, LATEX_MACRO, "textasciitilde", { "{\\textasciitilde}", "\\textasciitilde",   "\\~{}"            } }, /* Tilde in latex */
/* fix sticky spaces "~" in bibtex token cleaning--allows tokens to be parsed properly */

                                            /* Latin Capital A */
   { 192, LATEX_MACRO, "`A",   { "{\\`A}",   "\\`{A}",   "\\`A"  } }, /*               with grave */
   { 193, LATEX_MACRO, "'A",   { "{\\'A}",   "\\'{A}",   "\\'A"  } }, /*               with acute */
   { 194, LATEX_MACRO, "^A",   { "{\\^A}",   "\\^{A}",   "\\^A"  } }, /*               with circumflex */
   { 195, LATEX_MACRO, "~A",   { "{\\~A}",   "\\~{A}",   "\\~A"  } }, /*               with tilde */
   { 196, LATEX_MACRO, "\"A",  { "{\\\"A}",  "\\\"{A}",  "\\\"A" } }, /*               with diuresis */
   { 197, LATEX_MACRO, "AA",   { "{\\AA}",   "{\\r{A}}", "\\r{A}"} }, /*               with ring above */
   { 256, LATEX_MACRO, "={A}", { "{\\={A}}", "\\={A}",   "\\=A"  } }, /*               with macron */
   { 258, LATEX_MACRO, "u{A}", { "{\\u{A}}", "\\u{A}",   "\\u A" } }, /*               with breve */
   { 260, LATEX_MACRO, "k{A}", { "{\\k{A}}", "\\k{A}",   "\\k A" } }, /*               with ogonek */
   { 461, LATEX_MACRO, "v{A}", { "{\\v{A}}", "\\v{A}",   "\\v A" } }, /*               with caron */

                                           /* Latin Small a */
   { 224, LATEX_MACRO, "`a",   { "{\\`a}",   "\\`{a}",   "\\`a"  } }, /*               with grave */
   { 225, LATEX_MACRO, "'a",   { "{\\'a}",   "\\'{a}",   "\\'a"  } }, /*               with acute */
   { 226, LATEX_MACRO, "^a",   { "{\\^a}",   "\\^{a}",   "\\^a"  } }, /*               with circumflex */
   { 227, LATEX_MACRO, "~a",   { "{\\~a}",   "\\~{a}",   "\\~a"  } }, /*               with tilde */
   { 228, LATEX_MACRO, "\"a",  { "{\\\"a}",  "\\\"{a}",  "\\\"a" } }, /*               with diuresis */
   { 229, LATEX_MACRO, "aa",   { "{\\aa}",   "{\\r{a}}", "\\r{a}"} }, /*               with ring above */
   { 257, LATEX_MACRO, "={a}", { "{\\={a}}", "\\={a}",   "\\=a"  } }, /*               with macron */
   { 259, LATEX_MACRO, "u{a}", { "{\\u{a}}", "\\u{a}",   "\\u a" } }, /*               with breve */
   { 261, LATEX_MACRO, "k{a}", { "{\\k{a}}", "\\k{a}",   "\\k a" } }, /*               with ogonek */
   { 462, LATEX_MACRO, "v{a}", { "{\\v{a}}", "\\v{a}",   "\\v a" } }, /*               with caron */

   { 198, LATEX_MACRO, "AE",   { "{\\AE}",   "\\AE",     NULL    } }, /* Latin Capital AE */
   { 230, LATEX_MACRO, "ae",   { "{\\ae}",   "\\ae",     NULL    } }, /* Latin Small ae */

                                           /* Latin Capital C */
   { 199, LATEX_MACRO, "c{C}", { "{\\c{C}}", "\\c{C}",  "\\c c"  } }, /*               with cedilla */
   { 262, LATEX_MACRO, "'{C}", { "{\\'{C}}", "\\'{C}",  "\\'C"   } }, /*               with acute */
   { 264, LATEX_MACRO, "^{C}", { "{\\^{C}}", "\\^{C}",  "\\^C"   } }, /*               with circumflex */
   { 266, LATEX_MACRO, ".{C}", { "{\\.{C}}", "\\.{C}",  "\\.C"   } }, /*               with dot above */
   { 268, LATEX_MACRO, "v{C}", { "{\\v{C}}", "\\v{C}",  "\\v C"  } }, /*               with caron (hacek) */

                                           /* Latin Small c */
   { 231, LATEX_MACRO, "c{c}", { "{\\c{c}}", "\\c{c}",  "\\c C"  } }, /*               with cedilla*/
   { 263, LATEX_MACRO, "'{c}", { "{\\'{c}}", "\\'{c}",  "\\'c"   } }, /*               with acute */
   { 265, LATEX_MACRO, "^{c}", { "{\\^{c}}", "\\^{c}",  "\\^c"   } }, /*               with circumflex */
   { 267, LATEX_MACRO, ".{c}", { "{\\.{c}}", "\\.{c}",  "\\.c"   } }, /*               with dot above */
   { 269, LATEX_MACRO, "v{c}", { "{\\v{c}}", "\\v{c}",  "\\v c"  } }, /*               with caron (hacek) */

                                           /* Latin Capital D */
   { 270, LATEX_MACRO, "v{D}", { "{\\v{D}}", "\\v{D}",  "\\v D"  } }, /*               with caron */
   { 272, LATEX_MACRO, "DJ",   { "{\\DJ}",   NULL,      NULL     } }, /*               with stroke */

                                           /* Latin Small d */
   { 271, LATEX_MACRO, "v{d}", { "{\\v{d}}", "\\v{d}",  "\\v d"  } }, /*               with caron */
   { 273, LATEX_MACRO, "dj",   { "{\\dj}",   NULL,      NULL     } }, /*               with stroke */

                                           /* Latin Capital E */
   { 200, LATEX_MACRO, "`E",   { "{\\`E}",   "\\`{E}",  "\\`E"   } }, /*               with grave */
   { 201, LATEX_MACRO, "'E",   { "{\\'E}",   "\\'{E}",  "\\'E"   } }, /*               with acute */
   { 202, LATEX_MACRO, "^E",   { "{\\^E}",   "\\^{E}",  "\\^E"   } }, /*               with circumflex */
   { 203, LATEX_MACRO, "\"E",  { "{\\\"E}",  "\\\"{E}", "\\\"E"  } }, /*               with diuresis */
   { 274, LATEX_MACRO, "={E}", { "{\\={E}}", "\\={E}",  "\\=E"   } }, /*               with macron */
   { 276, LATEX_MACRO, "u{E}", { "{\\u{E}}", "\\u{E}",  "\\u E"  } }, /*               with breve */
   { 278, LATEX_MACRO, ".{E}", { "{\\.{E}}", "\\.{E}",  "\\.E"   } }, /*               with dot above */
   { 280, LATEX_MACRO, "k{E}", { "{\\k{E}}", "\\k{E}",  "\\k E"  } }, /*               with ogonek */
   { 282, LATEX_MACRO, "v{E}", { "{\\v{E}}", "\\v{E}",  "\\v E"  } }, /*               with caron */
 
                                           /* Latin Small e */
   { 232, LATEX_MACRO, "`e",   { "{\\`e}",   "\\`{e}",  "\\`e"   } }, /*               with grave */
   { 233, LATEX_MACRO, "'e",   { "{\\'e}",   "\\'{e}",  "\\'e"   } }, /*               with acute */
   { 234, LATEX_MACRO, "^e",   { "{\\^e}",   "\\^{e}",  "\\^e"   } }, /*               with circumflex */
   { 235, LATEX_MACRO, "\"e",  { "{\\\"e}",  "\\\"{e}", "\\\"e"  } }, /*               with diuresis */
   { 275, LATEX_MACRO, "={e}", { "{\\={e}}", "\\={e}",  "\\=e"   } }, /*               with macron */
   { 277, LATEX_MACRO, "u{e}", { "{\\u{e}}", "\\u{e}",  "\\u e"  } }, /*               with breve */
   { 279, LATEX_MACRO, ".{e}", { "{\\.{e}}", "\\.{e}",  "\\.e"   } }, /*               with dot above */
   { 281, LATEX_MACRO, "k{e}", { "{\\k{e}}", "\\k{e}",  "\\k e"  } }, /*               with ogonek */
   { 283, LATEX_MACRO, "v{e}", { "{\\v{e}}", "\\v{e}",  "\\v e"  } }, /*               with caron */

                                           /* Latin Capital G */
   { 284, LATEX_MACRO, "^{G}", { "{\\^{G}}", "\\^{G}",  "\\^G"   } }, /*               with circumflex */
   { 286, LATEX_MACRO, "u{G}", { "{\\u{G}}", "\\u{G}",  "\\u G"  } }, /*               with breve */
   { 288, LATEX_MACRO, ".{G}", { "{\\.{G}}", "\\.{G}",  "\\.G"   } }, /*               with dot above */
   { 290, LATEX_MACRO, "c{G}", { "{\\c{G}}", "\\c{G}",  "\\c G"  } }, /*               with cedilla */
   { 486, LATEX_MACRO, "v{G}", { "{\\v{G}}", "\\v{G}",  "\\v G"  } }, /*               with caron */
   { 500, LATEX_MACRO, "'{G}", { "{\\'{G}}", "\\'{G}",  "\\'G"   } }, /*               with acute */

                                           /* Latin Small g */
   { 285, LATEX_MACRO, "^{g}", { "{\\^{g}}", "\\^{g}",  "\\^g"   } }, /*               with circumflex */
   { 287, LATEX_MACRO, "u{g}", { "{\\u{g}}", "\\u{g}",  "\\u g"  } }, /*               with breve */
   { 289, LATEX_MACRO, ".{g}", { "{\\.{g}}", "\\.{g}",  "\\.g"   } }, /*               with dot above */
   { 291, LATEX_MACRO, "c{g}", { "{\\c{g}}", "\\c{g}",  "\\c g"  } }, /*               with cedilla */
   { 487, LATEX_MACRO, "v{g}", { "{\\v{g}}", "\\v{g}",  "\\v g"  } }, /*               with caron */
   { 501, LATEX_MACRO, "'{g}", { "{\\'{g}}", "\\'{g}",  "\\'g"   } }, /*               with acute */

                                           /* Latin Capital H */
   { 292, LATEX_MACRO, "^{H}", { "{\\^{H}}", "\\^{H}",  "\\^H"   } }, /*               with circumflex */
/* { 294, LATEX_MACRO, "",     { NULL,       NULL,      NULL     } },*//*              with stroke */

                                           /* Latin Capital h */
   { 293, LATEX_MACRO, "^{h}", { "{\\^{h}}", "\\^{h}",  "\\^h"   } }, /*               with circumflex */
/* { 295, LATEX_MACRO, "",     { NULL,       NULL,      NULL     } },*//*              with stroke */
 
                                           /* Latin Capital I */
   { 204, LATEX_MACRO, "`I",   { "{\\`I}",   "\\`{I}",  "\\`I"   } }, /*               with grave */
   { 205, LATEX_MACRO, "'I",   { "{\\'I}",   "\\'{I}",  "\\'I"   } }, /*               with acute */
   { 206, LATEX_MACRO, "^I",   { "{\\^I}",   "\\^{I}",  "\\^I"   } }, /*               with circumflex */
   { 207, LATEX_MACRO, "\"I",  { "{\\\"I}",  "\\\"{I}", "\\\"I"  } }, /*               with diuresis */
   { 296, LATEX_MACRO, "~{I}", { "{\\~{I}}", "\\~{I}",  "\\~I"   } }, /*               with tilde */
   { 298, LATEX_MACRO, "={I}", { "{\\={I}}", "\\={I}",  "\\=I"   } }, /*               with macron */
   { 300, LATEX_MACRO, "u{I}", { "{\\u{I}}", "\\u{I}",  "\\u I"  } }, /*               with breve */
   { 302, LATEX_MACRO, "k{I}", { "{\\k{I}}", "\\k{I}",  "\\k I"  } }, /*               with ogonek */
   { 304, LATEX_MACRO, ".{I}", { "{\\.{I}}", "\\.{I}",  "\\. I"  } }, /*               with dot above */
   { 463, LATEX_MACRO, "v{I}", { "{\\v{I}}", "\\v{I}",  "\\v I"  } }, /*               with caron */

                                           /* Latin Small i */
   { 236, LATEX_MACRO, "`i",   { "{\\`i}",   "\\`{i}",  "\\`i"   } }, /*               with grave */
   { 237, LATEX_MACRO, "'i",   { "{\\'i}",   "\\'{i}",  "\\'i"   } }, /*               with acute */
   { 238, LATEX_MACRO, "^i",   { "{\\^i}",   "\\^{i}",  "\\^i"   } }, /*               with circumflex */
   { 239, LATEX_MACRO, "\"i",  { "{\\\"i}",  "\\\"{i}", "\\\"i"  } }, /*               with diuresis */
   { 303, LATEX_MACRO, "k{i}", { "{\\k{i}}", "\\k{i}",  "\\k i"  } }, /*               with ogonek */
                                           /* Latex "\i" has no dot on "i" */
   { 305, LATEX_MACRO, "i",       { "{\\i}",      "\\i{}",    NULL       } }, /*    without dot above */
   { 236, LATEX_MACRO, "`{\\i}",  { "{\\`\\i}",   "\\`{\\i}", "\\`\\i"   } }, /*       with grave */
   { 237, LATEX_MACRO, "'{\\i}",  { "{\\'\\i}",   "\\'{\\i}", "\\'\\i"   } }, /*       with acute */
   { 238, LATEX_MACRO, "^{\\i}",  { "{\\^\\i}",   "\\^{\\i}", "\\^\\i"   } }, /*       with circumflex */
   { 239, LATEX_MACRO, "\"{\\i}", { "{\\\"\\i}",  "\\\"{\\i}","\\\"\\i"  } }, /*       with diuresis */
   { 297, LATEX_MACRO, "`{\\i}",  { "{\\~{\\i}}", "\\~{\\i}", "\\~\\i{}" } }, /*       with tilde */
   { 299, LATEX_MACRO, "={\\i}",  { "{\\={\\i}}", "\\={\\i}", "\\=\\i{}" } }, /*       with macron */
   { 301, LATEX_MACRO, "u{\\i}",  { "{\\u{\\i}}", "\\u{\\i}", "\\u\\i{}" } }, /*       with breve */
   { 464, LATEX_MACRO, "v{\\i}",  { "{\\v \\i{}}", "\\v \\i{}", NULL   } }, /*         with caron */

/* { 306, LATEX_MACRO, "",     { NULL,       NULL,      NULL     } },*/ /* Latin Capital IJ */
/* { 307, LATEX_MACRO, "",     { NULL,       NULL,      NULL     } },*/ /* Latin Small ij */

                                           /* Latin Capital J */
   { 308, LATEX_MACRO, "^{J}", { "{\\^{J}}", "\\^{J}",  "\\^J"   } }, /*               with circumflex */

                                           /* Latin Small j */
   { 309, LATEX_MACRO, "^{j}", { "{\\^{j}}", "\\^{j}",  "\\^j"   } }, /*               with circumflex */

                                           /* Latin Capital K */
   { 310, LATEX_MACRO, "c{K}", { "{\\c{K}}", "\\c{K}",  "\\c K"  } }, /*               with cedilla */
   { 488, LATEX_MACRO, "v{K}", { "{\\v{K}}", "\\v{K}",  "\\v K"  } }, /*               with caron */

                                           /* Latin Small k */
   { 311, LATEX_MACRO, "c{k}", { "{\\c{k}}", "\\c{k}",  "\\c k"  } }, /*               with cedilla */
   { 489, LATEX_MACRO, "v{k}", { "{\\v{k}}", "\\v{k}",  "\\v k"  } }, /*               with caron */

                                           /* Latin Capital L */
   { 313, LATEX_MACRO, "'{L}", { "{\\'{L}}", "\\'{L}",  "\\'L"   } }, /*               with acute */
   { 315, LATEX_MACRO, "c{L}", { "{\\c{L}}", "\\c{L}",  "\\c L"  } }, /*               with cedilla */
   { 317, LATEX_MACRO, "v{L}", { "{\\v{L}}", "\\v{l}",  "\\v L"  } }, /*               with caron */
   { 319, LATEX_COMBO, "{L\\hspace{-0.35em}$\\cdot$}", { "{L\\hspace{-0.35em}$\\cdot$}","L\\hspace{-0.35em}$\\cdot$", NULL } }, /*               with middle dot */
   { 321, LATEX_MACRO, "L",    { "{\\L}",   "{\\L{}}", "\\L{}"   } }, /*               with stroke */

                                           /* Latin Small l */
   { 314, LATEX_MACRO, "'{l}", { "{\\'{l}}", "\\'{l}",  "\\'l"   } }, /*               with acute */
   { 316, LATEX_MACRO, "c{l}", { "{\\c{l}}", "\\c{l}",  "\\c l"  } }, /*               with cedilla */
   { 318, LATEX_MACRO, "v{l}", { "{\\v{l}}", "\\v{l}",  "\\v l"  } }, /*               with caron */
   { 320, LATEX_COMBO, "{l$\\cdot$}", { "{l$\\cdot$}","l$\\cdot$", NULL } }, /*               with middle dot */
   { 322, LATEX_MACRO, "l",    { "{\\l}",    "{\\l{}}", "\\l{}"  } }, /*               with stroke */

                                           /* Latin Capital N */
   { 209, LATEX_MACRO, "~{N}", { "{\\~{N}}", "\\~{N}",  "\\~N"   } }, /*               with tilde */
   { 323, LATEX_MACRO, "'{N}", { "{\\'{N}}", "\\'{N}",  "\\'N"   } }, /*               with acute */
   { 325, LATEX_MACRO, "c{N}", { "{\\c{N}}", "\\c{N}",  "\\c N"  } }, /*               with cedilla */
   { 327, LATEX_MACRO, "v{N}", { "{\\v{N}}", "\\v{N}",  "\\v N"  } }, /*               with caron */

                                           /* Latin Small n */
   { 241, LATEX_MACRO, "~{n}", { "{\\~{n}}", "\\~{n}",  "\\~n"   } }, /*               with tilde */
   { 324, LATEX_MACRO, "'{n}", { "{\\'{n}}", "\\'{n}",  "\\'n"   } }, /*               with acute */
   { 326, LATEX_MACRO, "c{n}", { "{\\c{n}}", "\\c{n}",  "\\c N"  } }, /*               with cedilla */
   { 328, LATEX_MACRO, "v{n}", { "{\\v{n}}", "\\v{n}",  "\\v n"  } }, /*               with caron */
   { 329, LATEX_MACRO, "n",    { "\\n",      NULL,      NULL     } }, /*               preceeded by apostrophe */
 
                                           /* Latin Capital O */
   { 210, LATEX_MACRO, "`O",   { "{\\`O}",   "\\`{O}",  "\\`O"   } }, /*               with grave */
   { 211, LATEX_MACRO, "'O",   { "{\\'O}",   "\\'{O}",  "\\'O"   } }, /*               with acute */
   { 212, LATEX_MACRO, "^O",   { "{\\^O}",   "\\^{O}",  "\\^O"   } }, /*               with circumflex */
   { 213, LATEX_MACRO, "~O",   { "{\\~O}",   "\\~{O}",  "\\~O"   } }, /*               with tilde */
   { 214, LATEX_MACRO, "\"O",  { "{\\\"O}",  "\\\"{O}", "\\\"O"  } }, /*               with diaeresis */
   { 216, LATEX_MACRO, "O",    { "{\\O}",    "\\O",     NULL     } }, /*               with stroke */
   { 332, LATEX_MACRO, "={O}", { "{\\={O}}", "\\={O}", "\\=O"    } }, /*               with macron */
   { 334, LATEX_MACRO, "u{O}", { "{\\u{O}}", "\\u{O}", "\\u O"   } }, /*               with breve */
   { 336, LATEX_MACRO, "H{O}", { "{\\H{O}}", "\\H{O}", "\\H O"   } }, /*               with double acute */
   { 465, LATEX_MACRO, "v{O}", { "{\\v{O}}", "\\v{O}", "\\v O"   } }, /*               with caron */
   { 490, LATEX_MACRO, "k{O}", { "{\\k{O}}", "\\k{O}", "\\k O"   } }, /*               with ogonek */

                                           /* Latin Small o */
   { 242, LATEX_MACRO, "`o",   { "{\\`o}",   "\\`{o}",  "\\`o"   } }, /*               with grave */
   { 243, LATEX_MACRO, "'o",   { "{\\'o}",   "\\'{o}",  "\\'o"   } }, /*               with acute */
   { 244, LATEX_MACRO, "^o",   { "{\\^o}",   "\\^{o}",  "\\^o"   } }, /*               with circumflex */
   { 245, LATEX_MACRO, "~o",   { "{\\~o}",   "\\~{o}",  "\\~o"   } }, /*               with tilde */
   { 246, LATEX_MACRO, "\"o",  { "{\\\"o}",  "\\\"{o}", "\\\"o"  } }, /*               with diaeresis */
   { 248, LATEX_MACRO, "o",    { "{\\o}",    "\\o",     NULL     } }, /*               with stroke */
   { 333, LATEX_MACRO, "={o}", { "{\\={o}}", "\\={o}", "\\=o"    } }, /*               with macron */
   { 335, LATEX_MACRO, "u{o}", { "{\\u{o}}", "\\u{o}", "\\u o"   } }, /*               with breve */
   { 337, LATEX_MACRO, "H{o}", { "{\\H{o}}", "\\H{o}", "\\H o"   } }, /*               with double acute */
   { 466, LATEX_MACRO, "v{o}", { "{\\v{o}}", "\\v{o}", "\\v o"   } }, /*               with caron */
   { 491, LATEX_MACRO, "k{o}", { "{\\k{o}}", "\\k{o}", "\\k o"   } }, /*               with ogonek */

   { 338, LATEX_MACRO, "OE",   { "{\\OE}",   "\\OE",   NULL      } }, /* Latin Capital OE */
   { 339, LATEX_MACRO, "oe",   { "{\\oe}",   "\\oe",   NULL      } }, /* Latin Small oe */

                                           /* Latin Capital R */
   { 340, LATEX_MACRO, "'R",   { "{\\'{R}}", "\\'{R}", "\\'R"    } }, /*               with acute */
   { 342, LATEX_MACRO, "c{R}", { "{\\c{R}}", "\\c{R}", "\\c R"   } }, /*               with cedilla */
   { 344, LATEX_MACRO, "v{R}", { "{\\v{R}}", "\\v{R}", "\\v R"   } }, /*               with caron */
 
                                           /* Latin Small r */
   { 341, LATEX_MACRO, "'r",   { "{\\'{r}}", "\\'{r}", "\\'r"    } }, /*               with acute */
   { 343, LATEX_MACRO, "c{r}", { "{\\c{r}}", "\\c{r}", "\\c r"   } }, /*               with cedilla */
   { 345, LATEX_MACRO, "v{r}", { "{\\v{r}}", "\\v{r}", "\\v r"   } }, /*               with caron */

                                           /* Latin Capital S */
   { 346, LATEX_MACRO, "'{S}", { "{\\'{S}}", "\\'{S}", "\\'S"    } }, /*               with acute */
   { 348, LATEX_MACRO, "^{S}", { "{\\^{S}}", "\\^{S}", "\\^S"    } }, /*               with circumflex */
   { 350, LATEX_MACRO, "c{S}", { "{\\c{S}}", "\\c{S}", "\\c S"   } }, /*               with cedilla */
   { 352, LATEX_MACRO, "v{S}", { "{\\v{S}}", "\\v{S}", "\\v S"   } }, /*               with caron */

                                           /* Latin Small s */
   { 347, LATEX_MACRO, "'{s}", { "{\\'{s}}", "\\'{s}", "\\'s"    } }, /*               with acute */
   { 349, LATEX_MACRO, "^{s}", { "{\\^{s}}", "\\^{s}", "\\^s"    } }, /*               with circumflex */
   { 351, LATEX_MACRO, "c{s}", { "{\\c{s}}", "\\c{s}", "\\c s"   } }, /*               with cedilla */
   { 353, LATEX_MACRO, "v{s}", { "{\\v{s}}", "\\v{s}", "\\v s"   } }, /*               with caron */

                                           /* Latin Capital T */
   { 354, LATEX_MACRO, "c{T}", { "{\\c{T}}", "\\c{T}", NULL      } }, /*               with cedilla */
   { 356, LATEX_MACRO, "v{T}", { "{\\v{T}}", "\\v{T}", NULL      } }, /*               with caron */
/* { 358, LATEX_MACRO, "",     { NULL,       NULL,     NULL      } },*//*              with stroke */

                                           /* Latin Small t */
   { 355, LATEX_MACRO, "c{T}", { "{\\c{t}}", "\\c{t}", NULL      } }, /*               with cedilla */
   { 357, LATEX_MACRO, "v{T}", { "{\\v{t}}", "\\v{t}", NULL      } }, /*               with caron */
/* { 359, LATEX_MACRO, "",     { NULL,       NULL,     NULL      } },*//*              with stroke */

   { 223, LATEX_MACRO, "ss",   { "{\\ss}",   "\\ss",   NULL      } }, /* German sz ligature, "sharp s" */

                                           /* Latin Capital U */
   { 217, LATEX_MACRO, "`U",   { "{\\`U}",   "\\`{U}", "\\`U"    } }, /*               with grave */
   { 218, LATEX_MACRO, "'U",   { "{\\'U}",   "\\'{U}", "\\'U"    } }, /*               with acute */
   { 219, LATEX_MACRO, "^U",   { "{\\^U}",   "\\^{U}", "\\^U"    } }, /*               with circumflex */
   { 220, LATEX_MACRO, "\"U",  { "{\\\"U}",  "\\\"{U}","\\\"U"   } }, /*               with diaeresis */
   { 360, LATEX_MACRO, "~{U}", { "{\\~{U}}", "\\~{U}", "\\~U"    } }, /*               with tilde */
   { 362, LATEX_MACRO, "={U}", { "{\\={U}}", "\\={U}", "\\=U"    } }, /*               with macron */
   { 364, LATEX_MACRO, "u{U}", { "{\\u{U}}", "\\u{U}", "\\u U"   } }, /*               with breve */
   { 366, LATEX_MACRO, "r{U}", { "{\\r{U}}", "\\r{U}", "\\r U"   } }, /*               with ring above */
   { 368, LATEX_MACRO, "H{U}", { "{\\H{U}}", "\\H{U}", "\\H U"   } }, /*               with double acute */
   { 370, LATEX_MACRO, "k{U}", { "{\\k{U}}", "\\k{U}", "\\k U"   } }, /*               with ogonek */
   { 467, LATEX_MACRO, "v{U}", { "{\\v{U}}", "\\v{U}", "\\v U"   } }, /*               with caron */

                                           /* Latin Small u */
   { 249, LATEX_MACRO, "`u",   { "{\\`u}",   "\\`{u}", "\\`u"    } }, /*               with grave */
   { 250, LATEX_MACRO, "'u",   { "{\\'u}",   "\\'{u}", "\\'u"    } }, /*               with acute */
   { 251, LATEX_MACRO, "^u",   { "{\\^u}",   "\\^{u}", "\\^u"    } }, /*               with circumflex */
   { 252, LATEX_MACRO, "\"u",  { "{\\\"u}",  "\\\"{u}","\\\"u"   } }, /*               with diaeresis */
   { 361, LATEX_MACRO, "~{u}", { "{\\~{u}}", "\\~{u}", NULL      } }, /*               with tilde */
   { 363, LATEX_MACRO, "={u}", { "{\\={u}}", "\\={u}", "\\=u"    } }, /*               with macron */
   { 365, LATEX_MACRO, "u{u}", { "{\\u{u}}", "\\u{u}", "\\u u"   } }, /*               with breve */
   { 367, LATEX_MACRO, "r{u}", { "{\\r{u}}", "\\r{u}", "\\r u"   } }, /*               with ring above */
   { 369, LATEX_MACRO, "H{u}", { "{\\H{u}}", "\\H{u}", "\\H u"   } }, /*               with double acute */
   { 371, LATEX_MACRO, "k{u}", { "{\\k{u}}", "\\k{u}", "\\k u"   } }, /*               with ogonek */
   { 468, LATEX_MACRO, "v{u}", { "{\\v{u}}", "\\v{u}", "\\v u"   } }, /*               with caron */

                                           /* Latin Capital W */
   { 372, LATEX_MACRO, "^{W}", { "{\\^{W}}", "\\^{W}", "\\^W"    } }, /*               with circumflex */

                                           /* Latin Small w */
   { 373, LATEX_MACRO, "^{w}", { "{\\^{w}}", "\\^{w}", "\\^w"    } }, /*               with circumflex */

                                           /* Latin Capital Y */
   { 221, LATEX_MACRO, "'{Y}", { "{\\'{Y}}", "\\'{Y}", "\\'Y"    } }, /*               with acute */
   { 374, LATEX_MACRO, "^{Y}", { "{\\^{Y}}", "\\^{Y}", "\\^Y"    } }, /*               with circumflex */
   { 376, LATEX_MACRO, "\"{Y}",{ "{\\\"{Y}}","\\\"{Y}","\\\"Y"   } }, /*               with diaeresis */

                                           /* Latin Small y */
   { 253, LATEX_MACRO, "'y",   { "{\\'y}",  "\\'{y}", "\\'y"     } }, /*               with acute */
   { 255, LATEX_MACRO, "\"y",  { "{\\\"y}", "\\\"{y}","\\\"y"    } }, /*               with diaeresis */
   { 375, LATEX_MACRO, "^{y}", { "{\\^{y}}","\\^{y}", "\\^y"     } }, /*               with circumflex */

                                           /* Latin Capital Z */
   { 377, LATEX_MACRO, "'{Z}", { "{\\'{Z}}","\\'{Z}", "\\'Z"     } }, /*               with acute */
   { 379, LATEX_MACRO, ".{Z}", { "{\\.{Z}}","\\.{Z}", "\\.Z"     } }, /*               with dot above */
   { 381, LATEX_MACRO, "v{Z}", { "{\\v{Z}}","\\v{Z}", "\\v Z"    } }, /*               with caron */

                                           /* Latin Small z */
   { 378, LATEX_MACRO, "'{z}", { "{\\'{z}}","\\'{z}", "\\'z"     } }, /*               with acute */
   { 380, LATEX_MACRO, ".{z}", { "{\\.{z}}","\\.{z}", "\\.z"     } }, /*               with dot above */
   { 382, LATEX_MACRO, "v{z}", { "{\\v{z}}","\\v{z}", "\\v z"    } }, /*               with caron */


				/* Needs to be before \nu */
   { 8203,LATEX_MATH, "null",  { "$\\null$", "\\null", NULL      } }, /* No space &#x200B; */

   { 913, LATEX_MATH, "Alpha",   { "$\\Alpha$",   "\\Alpha",    NULL    } }, /*GREEK CAPITAL LETTERALPHA*/
   { 914, LATEX_MATH, "Beta",    { "$\\Beta$",    "\\Beta",     NULL    } }, /*GREEK CAPITAL LETTERBETA*/
   { 915, LATEX_MATH, "Gamma",   { "$\\Gamma$",   "\\Gamma",    NULL    } }, /*GREEK CAPITAL LETTERGAMMA*/
   { 916, LATEX_MATH, "Delta",   { "$\\Delta$",   "\\Delta",    NULL    } }, /*GREEK CAPITAL LETTERDELTA*/
   { 917, LATEX_MATH, "Epsilon", { "$\\Epsilon$", "\\Epsilon",  NULL    } }, /*GREEK CAPITAL LETTEREPSILON*/
   { 918, LATEX_MATH, "Zeta",    { "$\\Zeta$",     "\\Zeta",    NULL    } }, /*GREEK CAPITAL LETTERZETA*/
   { 919, LATEX_MATH, "Eta",     { "$\\Eta$",      "\\Eta",     NULL    } }, /*GREEK CAPITAL LETTERETA*/
   { 920, LATEX_MATH, "Theta",   { "$\\Theta$",    "\\Theta",   NULL    } }, /*GREEK CAPITAL LETTERTHETA*/
   { 921, LATEX_MATH, "Iota",    { "$\\Iota$",     "\\Iota",    NULL    } }, /*GREEK CAPITAL LETTERIOTA*/
   { 922, LATEX_MATH, "Kappa",   { "$\\Kappa$",    "\\Kappa",   NULL    } }, /*GREEK CAPITAL LETTERKAPPA*/
   { 923, LATEX_MATH, "Lambda",  { "$\\Lambda$",   "\\Lambda",  NULL    } }, /*GREEK CAPITAL LETTERLAMDA*/
   { 924, LATEX_MATH, "Mu",      { "$\\Mu$",       "\\Mu",      NULL    } }, /*GREEK CAPITAL LETTERMU*/
   { 925, LATEX_MATH, "Nu",      { "$\\Nu$",       "\\Nu",      NULL    } }, /*GREEK CAPITAL LETTERNU*/
   { 926, LATEX_MATH, "Xi",      { "$\\Xi$",       "\\Xi",      NULL    } }, /*GREEK CAPITAL LETTERXI*/
   { 927, LATEX_MATH, "Omicron", { "$\\Omicron$",  "\\Omicron", NULL    } }, /*GREEK CAPITAL LETTEROMICRON*/
   { 928, LATEX_MATH, "Pi",      { "$\\Pi$",       "\\Pi",      NULL    } }, /*GREEK CAPITAL LETTERPI*/
   { 929, LATEX_MATH, "Rho",     { "$\\Rho$",      "\\Rho",     NULL    } }, /*GREEK CAPITAL LETTERRHO*/
   { 931, LATEX_MATH, "Sigma",   { "$\\Sigma$",    "\\Sigma",   NULL    } }, /*GREEK CAPITAL LETTERSIGMA*/
   { 932, LATEX_MATH, "Tau",     { "$\\Tau$",      "\\Tau",     NULL    } }, /*GREEK CAPITAL LETTERTAU*/
   { 933, LATEX_MATH, "Upsilon", { "$\\Upsilon$",  "\\Upsilon", NULL    } }, /*GREEK CAPITAL LETTERUPSILON*/
   { 934, LATEX_MATH, "Phi",     { "$\\Phi$",      "\\Phi",     NULL    } }, /*GREEK CAPITAL LETTERPHI*/
   { 935, LATEX_MATH, "Chi",     { "$\\Chi$",      "\\Chi",     NULL    } }, /*GREEK CAPITAL LETTERCHI*/
   { 936, LATEX_MATH, "Psi",     { "$\\Psi$",      "\\Psi",     NULL    } }, /*GREEK CAPITAL LETTERPSI*/
   { 937, LATEX_MATH, "Omega",   { "$\\Omega$",    "\\Omega",   NULL    } }, /*GREEK CAPITAL LETTEROMEGA*/
   /* 902 = GREEK CAPITAL LETTER ALPHAWITHTONOS*/
   /* 904 = GREEK CAPITAL LETTER EPSILONWITHTONOS*/
   /* 905 = GREEK CAPITAL LETTER ETAWITHTONOS*/
   /* 938 = GREEK CAPITAL LETTER IOTAWITHDIALYTIKA*/
   /* 906 = GREEK CAPITAL LETTER IOTAWITHTONOS*/
   /* 908 = GREEK CAPITAL LETTER OMICRONWITHTONOS*/
   /* 939 = GREEK CAPITAL LETTER UPSILONWITHDIALYTIKA*/
   /* 910 = GREEK CAPITAL LETTER UPSILONWITHTONOS*/
   /* 911, = GREEK CAPITAL LETTER OMEGAWITHTONOS*/

   { 945, LATEX_MATH, "alpha",   { "$\\alpha$",    "\\alpha",   NULL    } }, /*GREEK SMALL LETTER ALPHA*/
   { 946, LATEX_MATH, "beta",    { "$\\beta$",     "\\beta",    NULL    } }, /*GREEK SMALL LETTER BETA*/
   { 968, LATEX_MATH, "psi",     { "$\\psi$",      "\\psi",     NULL    } }, /*GREEK SMALL LETTER PSI*/
   { 948, LATEX_MATH, "delta",   { "$\\delta$",    "\\delta",   NULL    } }, /*GREEK SMALL LETTER DELTA*/
   { 949, LATEX_MATH, "epsilon", { "$\\epsilon$",  "\\epsilon", NULL    } }, /*GREEK SMALL LETTER EPSILON*/
   { 966, LATEX_MATH, "phi",     { "$\\phi$",      "\\phi",     NULL    } }, /*GREEK SMALL LETTER PHI*/
   { 947, LATEX_MATH, "gamma",   { "$\\gamma$", "\\gamma", NULL }   }, /*GREEK SMALL LETTER GAMMA*/
   { 951, LATEX_MATH, "eta",     { "$\\eta$", "\\eta", NULL }     }, /*GREEK SMALL LETTER ETA*/
   { 953, LATEX_MATH, "iota",    { "$\\iota$", "\\iota", NULL }    }, /*GREEK SMALL LETTER IOTA*/
   { 958, LATEX_MATH, "xi",      { "$\\xi$", "\\xi", NULL }      }, /*GREEK SMALL LETTER XI*/
   { 954, LATEX_MATH, "kappa",   { "$\\kappa$", "\\kappa" , NULL }  }, /*GREEK SMALL LETTER KAPPA*/
   { 955, LATEX_MATH, "lambda",  { "$\\lambda$", "\\lambda", NULL }  }, /*GREEK SMALL LETTER LAMDA*/
   { 956, LATEX_MATH, "mu",      { "$\\mu$", "\\mu", NULL }      }, /*GREEK SMALL LETTER MU*/
   { 957, LATEX_MATH, "nu",      { "$\\nu$", "\\nu", NULL }      }, /*GREEK SMALL LETTER NU*/
   { 959, LATEX_MATH, "omicron", { "$\\omicron$", "\\omicron", NULL  }    }, /*GREEK SMALL LETTER OMICRON*/
   { 960, LATEX_MATH, "pi",      { "$\\pi$", "\\pi", NULL }      }, /*GREEK SMALL LETTER PI*/
   { 961, LATEX_MATH, "rho",     { "$\\rho$", "\\rho", NULL }     }, /*GREEK SMALL LETTER RHO*/
   { 963, LATEX_MATH, "sigma",   { "$\\sigma$", "\\sigma", NULL }   }, /*GREEK SMALL LETTER SIGMA*/
   { 964, LATEX_MATH, "tau",     { "$\\tau$", "\\tau", NULL }     }, /*GREEK SMALL LETTER TAU*/
   { 952, LATEX_MATH, "theta",   { "$\\theta$", "\\theta", NULL }   }, /*GREEK SMALL LETTER THETA*/
   { 969, LATEX_MATH, "omega",   { "$\\omega$", "\\omega", NULL }   }, /*GREEK SMALL LETTER OMEGA*/
   { 967, LATEX_MATH, "chi",     { "$\\chi$", "\\chi", NULL }     }, /*GREEK SMALL LETTER CHI*/
   { 965, LATEX_MATH, "upsilon", { "$\\upsilon$", "\\upsilon", NULL } }, /*GREEK SMALL LETTER UPSILON*/
   { 950, LATEX_MATH, "zeta",    { "$\\zeta$", "\\zeta", NULL }    },  /*GREEK SMALL LETTER ZETA*/
   /* 940 = GREEK SMALL LETTER ALPHAWITHTONOS*/
   /* 941 = GREEK SMALL LETTER EPSILONWITHTONOS*/
   /* 942 = GREEK SMALL LETTER ETAWITHTONOS */
   /* 912 = GREEK SMALL LETTER IOTAWITHDIALYTIKAANDTONOS*/
   /* 943 = GREEK SMALL LETTER IOTAWITHTONOS */
   /* 970 = GREEK SMALL LETTER IOTAWITHDIALYTIKA */
   /* 972 = GREEK SMALL LETTER OMICRONWITHTONOS*/
   /* 974 = GREEK SMALL LETTER OMEGAWITHTONOS*/
   /* 973 = GREEK SMALL LETTER UPSILONWITHTONOS*/
   /* 971 = GREEK SMALL LETTER UPSILONWITHDIALYTIKA*/
   /* 944 = GREEK SMALL LETTER UPSILONWITHDIALYTIKAANDTONOS*/

   { 181, LATEX_MACRO, "textmu", { "{\\textmu}", "\\textmu", "$\\mu$" } }, /* 181=micro sign, techically &#xB5; */

/* Make sure that these don't stomp on other latex things above */

   { 8242, LATEX_MACRO, "textasciiacutex",   { "{\\textasciiacutex}",   "\\textasciiacutex",     "$'$"                 } }, /* Prime symbol &#x2032; */
   { 180,  LATEX_MACRO, "textasciiacute",    { "{\\textasciiacute}",    "\\textasciiacute",      "\\'"                 } }, /* acute accent &#xB4; */
   { 8243, LATEX_MACRO, "textacutedbl",      { "{\\textacutedbl}",      "\\textacutedbl",        "$''$"                } }, /* Double prime &#x2033; */
   { 8245, LATEX_MACRO, "textasciigrave",    { "{\\textasciigrave}",    "\\textasciigrave",      "\\`"                 } }, /* Grave accent &#x2035; */
/* { 768,  LATEX_MACRO, "`",                 { "\\`",                   NULL,                    NULL                  } },*//* Grave accent &#x0300;--apply to next char */
/* { 769,  LATEX_MACRO, "'",                 { "\\'",                   NULL,                    NULL                  } },*//* Acute accent &#x0301;--apply to next char */

   { 8963, LATEX_MACRO, "textasciicircum",   { "{\\textasciicircum}",   "\\textasciicircum",     NULL                  } }, /* &#x2303; */
   { 184,  LATEX_MACRO, "textasciicedilla",  { "{\\textasciicedilla}",  "\\textasciicedilla",    NULL                  } }, /* cedilla &#xB8; */
   { 168,  LATEX_MACRO, "textasciidieresis", { "{\\textasciidieresis}", "\\textasciidieresis",   NULL                  } }, /* dieresis &#xA8; */
   { 175,  LATEX_MACRO, "textasciimacron",   { "{\\textasciimacron}",   "\\textasciimacron",     NULL                  } }, /* macron &#xAF; */

   { 8593, LATEX_MACRO, "textuparrow",       { "{\\textuparrow}",       "\\textuparrow",         NULL                  } }, /* Up arrow &#x2191; */
   { 8595, LATEX_MACRO, "textdownarrow",     { "{\\textdownarrow}",     "\\textdownarrow",       NULL                  } }, /* Down arrow &#x2193; */
   { 8594, LATEX_MACRO, "textrightarrow",    { "{\\textrightarrow}",    "\\textrightarrow",      NULL                  } }, /* Right arrow &#x2192; */
   { 8592, LATEX_MACRO, "textleftarrow",     { "{\\textleftarrow}",     "\\textleftarrow",       NULL                  } }, /* Left arrow &#x2190; */
   { 12296,LATEX_MACRO, "textlangle",        { "{\\textlangle}",        "\\textlangle",          NULL                  } }, /* L-angle &#x3008; */
   { 12297,LATEX_MACRO, "textrangle",        { "{\\textrangle}",        "\\textrangle",          NULL                  } }, /* L-angle &#x3009; */

   { 166,  LATEX_MACRO, "textbrokenbar",     { "{\\textbrokenbar}",     "\\textbrokenbar",       NULL                  } }, /* Broken vertical bar &#xA6; */
   { 167,  LATEX_MACRO, "textsection",       { "{\\textsection}",       "\\textsection",         "\\S{}"               } }, /* Section sign, &#xA7; */
   { 170,  LATEX_MACRO, "textordfeminine",   { "{\\textordfeminine}",   "\\textordfeminine",     "$^a$"                } }, /* &#xAA; */
   { 172,  LATEX_MACRO, "textlnot",          { "{\\textlnot}",          "\\textlnot",            NULL                  } }, /* Lnot &#xAC; */
   { 182,  LATEX_MACRO, "textparagraph",     { "{\\textparagraph}",     "\\textparagraph",       NULL                  } }, /* Paragraph sign &#xB6; */
   { 183,  LATEX_MACRO, "textperiodcentered",{ "{\\textperiodcentered}","\\textperiodcentered",  NULL                  } }, /* Period-centered &#xB7; */
   { 186,  LATEX_MACRO, "textordmasculine",  { "{\\textordmasculine}",  "\\textordmasculine",    NULL                  } }, /* &#xBA; */
   { 8214, LATEX_MACRO, "textbardbl",        { "{\\textbardbl}",        "\\textbardbl",          NULL                  } }, /* Double vertical bar &#x2016; */
   { 8224, LATEX_MACRO, "textdagger",        { "{\\textdagger}",        "\\textdagger",          NULL                  } }, /* Dagger &#x2020; */
   { 8225, LATEX_MACRO, "textdaggerdbl",     { "{\\textdaggerdbl}",     "\\textdaggerdbl",       NULL                  } }, /* Double dagger &x2021; */
   { 8226, LATEX_MACRO, "textbullet",        { "{\\textbullet}",        "\\textbullet",          NULL                  } }, /* Bullet &#x2022; */
   { 8494, LATEX_MACRO, "textestimated",     { "{\\textestimated}",     "\\textestimated",       NULL                  } }, /* Estimated &#x212E; */
   { 9526, LATEX_MACRO, "textopenbullet",    { "{\\textopenbullet}",    "\\textopenbullet",      NULL                  } }, /* &#x2536; */

   { 8220, LATEX_COMBO, "``",                { "``",                    "{\\textquotedblleft}",  "\\textquotedblleft"  } }, /* Opening double quote &#x201C; */
   { 8221, LATEX_COMBO, "''",                { "''",                    "{\\textquotedblright}", "\\textquotedblright" } }, /* Closing double quote &#x201D; */
   { 8216, LATEX_COMBO, "`",                 { "`",                     "{\\textquoteleft}",     "\\textquoteleft"     } }, /* Opening single quote &#x2018; */
   { 8217, LATEX_COMBO, "'",                 { "'",                     "{\\textquoteright}",    "\\textquoteright"    } }, /* Closing single quote &#x2019; */
   { 8261, LATEX_MACRO, "textlquill",        { "{\\textlquill}",        "\\textlquill",          NULL                  } }, /* Left quill &#x2045; */
   { 8262, LATEX_MACRO, "textrquill",        { "{\\textrquill}",        "\\textrquill",          NULL                  } }, /* Right quill &#x2046; */

   { 8212, LATEX_COMBO, "---",               { "---",                   "{\\textemdash}",        "\\textemdash"        } }, /* Em-dash &#x2014; */
   { 8211, LATEX_COMBO, "--",                { "--",                    "{\\textendash}",        "\\textendash"        } }, /* En-dash &#x2013; */
   { 8230, LATEX_MACRO, "ldots",             { "{\\ldots}",             "{\\textellipsis}",      "\\textellipsis"      } }, /* Ellipsis &#x2026; */

   { 8194, LATEX_MACRO, "enspace",           { "{\\enspace}",           "\\hspace{.5em}",        NULL                  } }, /* En-space &#x2002; */
   { 8195, LATEX_MACRO, "emspace",           { "{\\emspace}",           "\\hspace{1em}",         NULL                  } }, /* Em-space &#x2003; */
   { 8201, LATEX_MACRO, "thinspace",         { "{\\thinspace}",         NULL,                    NULL                  } }, /* Thin space &#x2009; */
   { 8203, LATEX_MACRO, "textnospace",       { "{\\textnospace}",       "\\textnospace",         NULL                  } }, /* No space &#x200B; */
   { 9251, LATEX_MACRO, "textvisiblespace",  { "{\\textvisiblespace}",  "\\textvisiblespace",    NULL                  } }, /* Visible space &#x2423; */

   { 215,  LATEX_MACRO, "texttimes",         { "{\\texttimes}",         "\\texttimes",           NULL                  } }, /* Multiplication symbol &#xD7; */
   { 247,  LATEX_MACRO, "textdiv",           { "{\\textdiv}",           "\\textdiv",             NULL                  } }, /* Division symbol &#xF7; */
   { 177,  LATEX_MACRO, "textpm",            { "{\\textpm}",            "\\textpm",              NULL                  } }, /* Plus-minus character &#B1; */
   { 189,  LATEX_MACRO, "textonehalf",       { "{\\textonehalf}",       "\\textonehalf",         NULL                  } }, /* Vulgar fraction one half &#xBD; */
   { 188,  LATEX_MACRO, "textonequarter",    { "{\\textonequarter}",    "\\textonequarter",      NULL                  } }, /* Vulgar fraction one quarter &#xBD; */
   { 190,  LATEX_MACRO, "textthreequarters", { "{\\textthreequarters}", "\\textthreequarters",   NULL                  } }, /* Vulgar fraction three quarters &#xBE; */
   { 8240, LATEX_MACRO, "texttenthousand",   { "{\\texttenthousand}",   "\\texttenthousand",     NULL                  } }, /* Per thousand sign &#x2030; */
   { 8241, LATEX_MACRO, "textpertenthousand",{ "{\\textpertenthousand}","\\textpertenthousand",  NULL                  } }, /* Per ten thousand sign &#x2031;*/
   { 8260, LATEX_MACRO, "textfractionssolidus",{"{\\textfractionsolidus}", "\\textfractionsolidus", NULL               } }, /* &x8260; */
   { 8451, LATEX_MACRO, "textcelcius",       { "{\\textcelcius}",       "\\textcelcius",         NULL                  } }, /* Celcicus &#x2103; */
   { 8470, LATEX_MACRO, "textnumero",        { "{\\textnumero}",        "\\textnumero",          NULL                  } }, /* Numero symbol &#x2116; */
   { 8486, LATEX_MACRO, "textohm",           { "{\\textohm}",           "\\textohm",             NULL                  } }, /* Ohm symbol &#x2126; */
   { 8487, LATEX_MACRO, "textmho",           { "{\\textmho}",           "\\textmho",             NULL                  } }, /* Mho symbol &#x2127; */
   { 8730, LATEX_MACRO, "textsurd",          { "{\\textsurd}",          "\\textsurd",            NULL                  } }, /* &#x221A; */

   { 185,  LATEX_MACRO, "textonesuperior",   { "{\\textonesuperior}",   "\\textonesuperior",     "$^1$"                } }, /*Superscript 1 &#xB9; */
   { 178,  LATEX_MACRO, "texttwosuperior",   { "{\\texttwosuperior}",   "\\texttwosuperior",     "$^2$"                } }, /*Superscript 2 &#xB2; */
   { 179,  LATEX_MACRO, "textthreesuperior", { "{\\textthreesuperior}", "\\textthreesuperior",   "$^3$"                } }, /*Superscript 3 &#xB3; */

   { 161,  LATEX_MACRO, "textexclamdown",    { "{\\textexclamdown}",    "\\textexclamdown",      NULL                  } }, /* Inverted exclamation mark &#xA1;*/
   { 191,  LATEX_MACRO, "textquestiondown",  { "{\\textquestiondown}",  "\\textquestiondown",    NULL                  } }, /* Inverted question mark &#xBF; */

   { 162,  LATEX_MACRO, "textcent",          { "{\\textcent}",          "\\textcent",            NULL                  } }, /* Cent sign &#xA2; */
   { 163,  LATEX_MACRO, "textsterling",      { "{\\textsterling}",      "\\textsterling",        "\\pounds"            } }, /* Pound sign &#xA3; */
   { 165,  LATEX_MACRO, "textyen",           { "{\\textyen}",           "\\textyen",             NULL                  } }, /* Yen sign &#xA5; */
   { 402,  LATEX_MACRO, "textflorin",        { "{\\textflorin}",        "\\textflorin",          NULL                  } }, /* Florin sign &#x192; */
   { 3647, LATEX_MACRO, "textbaht",          { "{\\textbaht}",          "\\textbaht",            NULL                  } }, /* Thai currency &#xE3F; */
   { 8355, LATEX_MACRO, "textfrenchfranc",   { "{\\textfrenchfranc}",   "\\textfrenchfranc",     NULL                  } }, /* French franc &#x20A3; */
   { 8356, LATEX_MACRO, "textlira",          { "{\\textlira}",          "\\textlira",            NULL                  } }, /* Lira &#x20A4; */
   { 8358, LATEX_MACRO, "textnaira",         { "{\\textnaira}",         "\\textnaria",           NULL                  } }, /* Naira &#x20A6; */
   { 8361, LATEX_MACRO, "textwon",           { "{\\textwon}",           "\\textwon",             NULL                  } }, /* &#x20A9; */
   { 8363, LATEX_MACRO, "textdong",          { "{\\textdong}",          "\\textdong",            NULL                  } }, /* Vietnamese currency &#x20AB; */
   { 8364, LATEX_MACRO, "texteuro",          { "{\\texteuro}",          "\\texteuro",            NULL                  } }, /* Euro sign */

   { 169,  LATEX_MACRO, "textcopyright",     { "{\\textcopyright}",     "\\textcopyright",       NULL                  } }, /* Copyright (C) &#xA9; */
   { 175,  LATEX_MACRO, "textregistered",    { "{\\textregistered}",    "\\textregistered",      NULL                  } }, /* Registered sign (R) &#xAF;*/
   { 8482, LATEX_MACRO, "texttrademark",     { "{\\texttrademark}",     "\\texttrademark",       "$^{TM}$"             } }, /* Trademark (TM) &#x2122; */
   { 8480, LATEX_MACRO, "textservicemark",   { "{\\textservicemark}",   "\\textservicemark",     "$^{SM}$"             } }, /* Servicemark (SM) &#x2120;*/
   { 8471, LATEX_MACRO, "textcircledP",      { "{\\textcircledP}",      "\\textcircledP",        NULL                  } }, /* Circled P &#2117; */

};

static int nlatex_chars = sizeof(latex_chars)/sizeof(struct latex_chars);

/* latex2char()
 *
 *   Use the latex_chars[] lookup table to determine if any character
 *   is a special LaTeX code.  Note that if it is, then the equivalency
 *   is a Unicode character and we need to flag (by setting *unicode to 1)
 *   that we know the output is Unicode.  Otherwise, we set *unicode to 0,
 *   meaning that the output is whatever character set was given to us
 *   (which could be Unicode, but is not necessarily Unicode).
 *
 */
unsigned int
latex2char( char *s, unsigned int *pos, int *unicode )
{
	unsigned int value;
//	char *p, *q[3];
	int i, j, len;
	char *p;

	p = &( s[*pos] );
	value = (unsigned char) *p;
	if ( value=='{' || value=='\\' || value=='~' || 
	     value=='$' || value=='\'' || value=='`' || 
	     value=='-' || value=='^' ) {
//		if ( *p=='\\' && ( *p=='{' || *p=='}' ) ) {
//		} else {
		for ( i=0; i<nlatex_chars; ++i ) {
//			q[0] = latex_chars[i].bib1;
//			l[0] = strlen( q[0] );
//			q[1] = latex_chars[i].bib2;
//			l[1] = strlen( q[1] );
//			q[2] = latex_chars[i].bib3;
//			l[2] = strlen( q[2] );
			for ( j=0; j<3; ++j ) {
				if ( latex_chars[i].variant[j] == NULL ) continue;
				len = strlen( latex_chars[i].variant[j] );
				if ( !strncmp( p, latex_chars[i].variant[j], len ) ) {
					*pos = *pos + len;
					*unicode = 1;
					return latex_chars[i].unicode;
				}
			}
		}
//		}
	}
	*unicode = 0;
	*pos = *pos + 1;
	return value;
}

void
uni2latex( unsigned int ch, char buf[], int buf_size )
{
	int i, j, n;

	if ( buf_size==0 ) return;

	buf[0] = '?';
	buf[1] = '\0';

	if ( ch==' ' ) {
		buf[0] = ' '; /*special case to avoid &nbsp;*/
		return;
	}

	for ( i=0; i<nlatex_chars; ++i ) {
		if ( ch == latex_chars[i].unicode ) {
			n = 0;

			if ( latex_chars[i].type == LATEX_MACRO ) {
				if ( n < buf_size ) buf[n++] = '{';
				if ( n < buf_size ) buf[n++] = '\\';
			}
			else if ( latex_chars[i].type == LATEX_MATH ) {
				if ( n < buf_size ) buf[n++] = '$';
				if ( n < buf_size ) buf[n++] = '\\';
			}

			j = 0;
			while ( latex_chars[i].out[j] ) {
				if ( n < buf_size ) buf[n++] = latex_chars[i].out[j];
				j++;
			}

			if ( latex_chars[i].type == LATEX_MACRO ) {
				if ( n < buf_size ) buf[n++] = '}';
			}
			else if ( latex_chars[i].type == LATEX_MATH ) {
				if ( n < buf_size ) buf[n++] = '$';
			}

			if ( n < buf_size ) buf[n] = '\0';
			else buf[ buf_size-1 ] = '\0';

			return;
		}
	}

	if ( ch < 128 ) buf[0] = (char)ch;
}

