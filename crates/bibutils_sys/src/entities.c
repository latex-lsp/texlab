/*
 * entities.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <string.h>
#include <ctype.h>
#include "entities.h"

/* HTML 4.0 entities */

typedef struct entities {
	char html[20];
	unsigned int unicode;
} entities;

entities html_entities[] = {
	/* Special Entities */
	{ "&quot;",     34 },  /* quotation mark */
	{ "&amp;",      38 },  /* ampersand */
	{ "&apos;",     39 },  /* apostrophe (note not defined in HTML) */
	{ "&lpar;",     40 },  /* left parenthesis */
	{ "&rpar;",     41 },  /* right parenthesis */
	{ "&hyphen;",   45 },  /* hyphen */
	{ "&lt;",       60 },  /* less-than sign */
	{ "&gt;",       62 },  /* greater-than sign */
	{ "&quest;",    63 },  /* question mark */
	{ "&OElig;",   338 },  /* Latin cap ligature OE */
	{ "&oelig;",   339 },  /* Latin small ligature OE */
	{ "&Scaron;",  352 },  /* Latin cap S with caron */
	{ "&scaron;",  353 },  /* Latin cap S with caron */
	{ "&Yuml;",    376 },  /* Latin cap y with diaeresis */
	{ "&circ;",    710 },  /* modifier letter circumflex */
	{ "&tilde;",   732 },  /* small tilde */
	{ "&ensp;",   8194 }, /* en space */
	{ "&emsp;",   8195 }, /* em space */
	{ "&thinsp;", 8201 }, /* thin space */
	{ "&zwnj;",   8204 }, /* zero width non-joiner */
	{ "&zwj;",    8205 }, /* zero width joiner */
	{ "&lrm;",    8206 }, /* left-to-right mark */
	{ "&rlm;",    8207 }, /* right-to-left mark */
	{ "&ndash;",  8211 }, /* en dash */
	{ "&mdash;",  8212 }, /* em dash */
	{ "&lsquo;",  8216 }, /* left single quotation mark */
	{ "&rsquo;",  8217 }, /* right single quot. mark */
	{ "&sbquo;",  8218 }, /* single low-9 quot. mark */
	{ "&ldquo;",  8220 }, /* left double quot. mark */
	{ "&rdquo;",  8221 }, /* right double quot. mark */
	{ "&bdquo;",  8222 }, /* double low-9 quot. mark */
	{ "&dagger;", 8224 }, /* dagger */
	{ "&Dagger;", 8225 }, /* double dagger */
	{ "&permil;", 8240 }, /* per mille sign */
	{ "&lsaquo;", 8249 }, /* sin. left angle quot mark */
	{ "&rsaquo;", 8250 }, /* sin. right angle quot mark */
	{ "&euro;",   8364 }, /* euro sign */
	/* Symbols and Greek characters */
	{ "&fnof;",    402 }, /* small f with hook = function */
	{ "&Alpha;",   913 }, /* capital alpha */
	{ "&Beta;",    914 }, /* capital beta */
	{ "&Gamma;",   915 }, /* capital gamma */
	{ "&Delta;",   916 }, /* capital delta */
	{ "&Epsilon;", 917 }, /* capital epsilon */
	{ "&Zeta;",    918 }, /* capital zeta */
	{ "&Eta;",     919 }, /* capital eta */
	{ "&Theta;",   920 }, /* capital theta */
	{ "&Iota;",    921 }, /* capital iota */
	{ "&Kappa;",   922 }, /* capital kappa */
	{ "&Lambda;",  923 }, /* capital lambda */
	{ "&Mu;",      924 }, /* capital mu */
	{ "&Nu;",      925 }, /* capital nu */
	{ "&Xi;",      926 }, /* capital xi */
	{ "&Omicron;", 927 }, /* capital omicron */
	{ "&Pi;",      928 }, /* capital pi */
	{ "&Rho;",     929 }, /* capital rho */
	{ "&Sigma;",   931 }, /* capital sigma */
	{ "&Tau;",     932 }, /* capital tau */
	{ "&Upsilon;", 933 }, /* capital upsilon */
	{ "&Phi;",     934 }, /* capital phi */
	{ "&Chi;",     935 }, /* capital chi */
	{ "&Psi;",     936 }, /* capital psi */
	{ "&Omega;",   937 }, /* capital omega */
	{ "&alpha;",   945 }, /* small alpha */
	{ "&beta;",    946 }, /* small beta */
	{ "&gamma;",   947 }, /* small gamma */
	{ "&delta;",   948 }, /* small delta */
	{ "&epsilon;", 949 }, /* small epsilon */
	{ "&zeta;",    950 }, /* small zeta */
	{ "&eta;",     951 }, /* small eta */
	{ "&theta;",   952 }, /* small theta */
	{ "&iota;",    953 }, /* small iota */
	{ "&kappa;",   954 }, /* small kappa */
	{ "&lambda;",  955 }, /* small lambda */
	{ "&mu;",      956 }, /* small mu */
	{ "&nu;",      957 }, /* small nu */
	{ "&xi;",      958 }, /* small xi */
	{ "&omicron;", 959 }, /* small omicron */
	{ "&pi;",      960 }, /* small pi */
	{ "&rho;",     961 }, /* small rho */
	{ "&sigmaf;",  962 }, /* small final sigma */
	{ "&sigma;",   963 }, /* small simga */
	{ "&tau;",     964 }, /* small tau */
	{ "&upsilon;", 965 }, /* small upsilon */
	{ "&phi;",     966 }, /* small phi */
	{ "&chi;",     967 }, /* small chi */
	{ "&psi;",     968 }, /* small psi */
	{ "&omega;",   969 }, /* small omega */
	{ "&thetasym;",977 }, /* small theta symbol */
	{ "&upsih;",   978 }, /* small upsilon with hook */
	{ "&piv;",     982 }, /* pi symbol */
	{ "&bull;",   8226 }, /* bullet = small blk circle */
	{ "&hellip;", 8230 }, /* horizontal ellipsis */
	{ "&prime;",  8242 }, /* prime = minutes = feet */
	{ "&Prime;",  8243 }, /* double prime */
	{ "&oline;",  8254 }, /* overline */
	{ "&frasl;",  8260 }, /* fraction slash */
	{ "&weierp;", 8472 }, /* Weierstrass p = power set */
	{ "&image;",  8465 }, /* imaginary part-black cap I */
	{ "&real;",   8476 }, /* real part-black cap R */
	{ "&trade;",  8482 }, /* trademark sign */
	{ "&alefsym;",8501 }, /* alef symbol */
	{ "&larr;",   8592 }, /* left arrow */
	{ "&uarr;",   8593 }, /* up arrow */
	{ "&rarr;",   8594 }, /* right arrow */
	{ "&darr;",   8595 }, /* down arrow */
	{ "&harr;",   8596 }, /* left/right arrow */
	{ "&crarr;",  8629 }, /* down arrow with corner left */
	{ "&lArr;",   8656 }, /* left double arrow */
	{ "&uArr;",   8657 }, /* up double arrow */
	{ "&rArr;",   8658 }, /* up double arrow */
	{ "&dArr;",   8659 }, /* up double arrow */
	{ "&hArr;",   8660 }, /* up double arrow */
	{ "&forall;", 8704}, /* for all */
	{ "&part;",   8706}, /* partial differential */
	{ "&exist;",  8707}, /* there exists */
	{ "&empty;",  8709}, /* empty set */
	{ "&nabla;",  8711}, /* nabla=backwards difference */
	{ "&isin;",   8712}, /* element of */
	{ "&notin;",  8713}, /* not an element of */
	{ "&ni;",     8715}, /* contains as member */
	{ "&prod;",   8719}, /* n-ary product */
	{ "&sum;",    8721}, /* n-ary summation */
	{ "&minus;",  8722}, /* minuss sign */
	{ "&lowast;", 8727}, /* asterisk operator */
	{ "&radic;",  8730}, /* square root */
	{ "&prop;",   8733}, /* proportional to */
	{ "&infin;",  8734}, /* infinity */
	{ "&ang;",    8736}, /* angle */
	{ "&and;",    8743}, /* logical and */
	{ "&or;",     8744}, /* logical or */
	{ "&cap;",    8745}, /* intersection */
	{ "&cup;",    8746}, /* union */
	{ "&int;",    8747}, /* integral */
	{ "&there4;", 8756}, /* therefore */
	{ "&sim;",    8764}, /* tilde operator */
	{ "&cong;",   8773}, /* approximately equal to */
	{ "&asymp;",  8776}, /* asymptotic to */
	{ "&ne;",     8800}, /* not equal to */
	{ "&equiv;",  8801}, /* identical to */
	{ "&le;",     8804}, /* less-than or equal to */
	{ "&ge;",     8805}, /* greater-than or equal to */
	{ "&sub;",    8834}, /* subset of */
	{ "&sup;",    8835}, /* superset of */
	{ "&nsub;",   8836}, /* not a subset of */
	{ "&sube;",   8838}, /* subset of or equal to */
	{ "&supe;",   8839}, /* superset of or equal to */
	{ "&oplus;",  8853}, /* circled plus = direct sum */
	{ "&otimes;", 8855}, /* circled times = vec prod */
	{ "&perp;",   8869}, /* perpendicular */
	{ "&sdot;",   8901}, /* dot operator */
	{ "&lceil;",  8968}, /* left ceiling */
	{ "&rceil;",  8969}, /* right ceiling */
	{ "&lfloor;", 8970}, /* left floor */
	{ "&rfloor;", 8971}, /* right floor */
	{ "&lang;",   9001}, /* left angle bracket */
	{ "&rang;",   9002}, /* right angle bracket */
	{ "&loz;",    9674}, /* lozenge */
	{ "&spades;", 9824}, /* spades */
	{ "&clubs;",  9827}, /* clubs */
	{ "&hearts;", 9829}, /* hearts */
	{ "&diams;",  9830}, /* diamonds */
	/* Latin-1 */
	{ "&nbsp;",    32 },  /* non-breaking space */
	{ "&iexcl;",  161 },  /* inverted exclamation mark */
	{ "&cent;",   162 },  /* cent sign */
	{ "&pound;",  163 },  /* pound sign */
	{ "&curren;", 164 },  /* currency sign */
	{ "&yen;",    165 },  /* yen sign */
	{ "&brvbar;", 166 },  /* broken vertical bar */
	{ "&sect;",   167 },  /* section sign */
	{ "&uml;",    168 },  /* diaeresis - spacing diaeresis */
	{ "&copy;",   169 },  /* copyright sign */
	{ "&ordf;",   170 },  /* feminine ordinal indicator */
	{ "&laquo;",  171 },  /* left-pointing guillemet */
	{ "&not;",    172 },  /* not sign */
	{ "&shy;",    173 },  /* soft (discretionary) hyphen */
	{ "&reg;",    174 },  /* registered sign */
	{ "&macr;",   175 },  /* macron = overline */
	{ "&deg;",    176 },  /* degree sign */
	{ "&plusmn;", 177 },  /* plus-minus sign */
	{ "&sup2;",   178 },  /* superscript two */
	{ "&sup3;",   179 },  /* superscript three */
	{ "&acute;",  180 },  /* acute accent = spacing acute */
	{ "&micro;",  181 },  /* micro sign */
	{ "&para;",   182 },  /* pilcrow (paragraph) sign */
	{ "&middot;", 183 },  /* middle dot (georgian comma) */
	{ "&cedil;",  184 },  /* cedilla = spacing cedilla */
	{ "&sup1;",   185 },  /* superscript one */
	{ "&ordm;",   186 },  /* masculine ordinal indicator */
	{ "&raquo;",  187 },  /* right pointing guillemet */
	{ "&frac14;", 188 },  /* 1/4 */
	{ "&frac12;", 189 },  /* 1/2 */
	{ "&frac34;", 190 },  /* 3/4 */
	{ "&iquest;", 191 },  /* inverted question mark */
	{ "&Agrave;", 192 },  /* cap A with grave */
	{ "&Aacute;", 193 },  /* cap A with acute */
	{ "&Acirc;",  194 },  /* cap A with circumflex */
	{ "&Atilde;", 195 },  /* cap A with tilde */
	{ "&Auml;",   196 },  /* cap A with diaeresis */
	{ "&Aring;",  197 },  /* cap A with ring */
	{ "&AElig;",  198 },  /* cap AE ligature */
	{ "&Ccedil;", 199 },  /* cap C with cedilla */
	{ "&Egrave;", 200 },  /* cap E with grave */
	{ "&Eacute;", 201 },  /* cap E with acute */
	{ "&Ecirc;",  202 },  /* cap E with circumflex */
	{ "&Euml;",   203 },  /* cap E with diaeresis */
	{ "&Igrave;", 204 },  /* cap I with grave */
	{ "&Iacute;", 205 },  /* cap I with acute */
	{ "&Icirc;",  206 },  /* cap I with circumflex */
	{ "&Iuml;",   207 },  /* cap I with diaeresis */
	{ "&ETH;",    208 },  /* cap letter ETH */
	{ "&Ntilde;", 209 },  /* cap N with tilde */
	{ "&Ograve;", 210 },  /* cap O with grave */
	{ "&Oacute;", 211 },  /* cap O with acute */
	{ "&Ocirc;",  212 },  /* cap O with circumflex */
	{ "&Otilde;", 213 },  /* cap O with tilde */
	{ "&Ouml;",   214 },  /* cap O with diaeresis */
	{ "&times;",  215 },  /* multiplication sign */
	{ "&Oslash;", 216 },  /* cap O with stroke */
	{ "&Ugrave;", 217 },  /* cap U with grave */
	{ "&Uacute;", 218 },  /* cap U with acute */
	{ "&Ucirc;",  219 },  /* cap U with circumflex */
	{ "&Uuml;",   220 },  /* cap U with diaeresis */
	{ "&Yacute;", 221 },  /* cap Y with acute */
	{ "&THORN;",  222 },  /* cap letter THORN */
	{ "&szlig;",  223 },  /* small sharp s = ess-zed */
	{ "&agrave;", 224 },  /* small a with grave */
	{ "&aacute;", 225 },  /* small a with acute */
	{ "&acirc;",  226 },  /* small a with cirucmflex */
	{ "&atilde;", 227 },  /* small a with tilde */
	{ "&amul;",   228 },  /* small a with diaeresis */
	{ "&aring;",  229 },  /* small a with ring */
	{ "&aelig;",  230 },  /* small ligature ae */
	{ "&ccedil;", 231 },  /* small c with cedilla */
	{ "&egrave;", 232 },  /* small e with grave */
	{ "&eacute;", 233 },  /* small e with acute */
	{ "&ecirc;",  234 },  /* small e with circumflex */
	{ "&emul;",   235 },  /* small e with diaeresis */
	{ "&igrave;", 236 },  /* small i with grave */
	{ "&iacute;", 237 },  /* small i with acute */
	{ "&icirc;",  238 },  /* small i with circumflex */
	{ "&iuml;",   239 },  /* small i with diaeresis */
	{ "&eth;",    240 },  /* latin small letter eth */
	{ "&ntilde;", 241 },  /* small n with tilde */
	{ "&ograve;", 242 },  /* small o with grave */
	{ "&oacute;", 243 },  /* small o with acute */
	{ "&ocirc;",  244 },  /* small o with circumflex */
	{ "&otilde;", 245 },  /* small o with tilde */
	{ "&ouml;",   246 },  /* small o with diaeresis */
	{ "&divide;", 247 },  /* division sign */
	{ "&oslash;", 248 },  /* small o with slash */
	{ "&ugrave;", 249 },  /* small u with grave */
	{ "&uacute;", 250 },  /* small u with acute */
	{ "&ucirc;",  251 },  /* small u with circumflex */
	{ "&uuml;",   252 },  /* small u with diaeresis */
	{ "&yacute;", 253 },  /* small y with acute */
	{ "&thorn;",  254 },  /* latin small letter thorn */
	{ "&yuml;",   255 },  /* small y with diaeresis */
};


static unsigned int
decode_html_entity( char *s, unsigned int *pi, int *err )
{
	int nhtml_entities = sizeof( html_entities ) / sizeof( entities );
	char *e;
	int i, n=-1, len;
	for ( i=0; i<nhtml_entities && n==-1; ++i ) {
		e = &(html_entities[i].html[0]);
		len = strlen( e );
		if ( !strncasecmp( &(s[*pi]), e, len ) ) {
			n = i;
			*pi += len;
		}
	}
	if ( n==-1 ) {
		*err = 1;
		return '&';
	} else {
		*err = 0;
		return html_entities[n].unicode;
	}
}


/*
 * decode decimal entity
 *
 *    extract a decimal entity from &#NNNN;
 *    s[*pi] points to the '&' character
 */
static unsigned int
decode_decimal_entity( char *s, unsigned int *pi, int *err )
{
	unsigned int c = 0, d;
	int i = *pi, j = 2;
	while ( isdigit( (unsigned char)s[i+j] ) ) {
		d = s[i+j] - '0';
		c = 10 * c + d;
		j++;
	}
	if ( s[i+j]!=';' ) *err = 1;
	else *pi = i+j+1;
	return c;
}

/*
 * decode hex entity
 *
 *    extract a hex entity from &#xNNNN;
 *    s[*pi] points to the '&' character
 */
static unsigned int
decode_hex_entity( char *s, unsigned int *pi, int *err )
{
	unsigned int c = 0, d;
	int i = *pi, j = 3;
	while ( isxdigit( (unsigned char)s[i+j] ) ) {
		if ( isdigit( (unsigned char)s[i+j] ) ) d = s[i+j]-'0';
		else d = toupper((unsigned char)s[i+j])-'A' + 10;
		c = 16 * c + d;
		j++;
	}
	if ( s[i+j]!=';' ) *err = 1;
	else *pi = i+j+1;
	return c;
}

/*
 * decode numeric entity
 *
 *    extract a numeric entity from &#NNN; or &#xNNNN;
 *
 *    In XML, the "x" in hexadecimal entries should be lowercase,
 *    but we'll be generous and accept "X" as well.
 */
static unsigned int
decode_numeric_entity( char *s, unsigned int *pi, int *err )
{
	unsigned int c;
	*err = 0;
	if ( s[*pi+2]!='x' && s[*pi+2]!='X' ) c = decode_decimal_entity( s, pi, err );
	else c = decode_hex_entity( s, pi, err );
	if ( *err ) {
		*pi = *pi + 1;
		c = '&';
	}
	return c;
}

/*
 * decode entity
 *    extract entity from  &mmmm;
 *
 * where &mmmm; is one of
 * - &#nnnn; is code point in decimal form
 * - &#xhhhh; is code point in hexadecimal form (note "x" is lowercase in XML)
 * - &mmmm; corresponds to a pre-defined XML entity, e.g. &quote for quotations
 *
 */
unsigned int
decode_entity( char *s, unsigned int *pi, int *unicode, int *err )
{
	unsigned int c = '&';
	*unicode = 0;

	if ( s[*pi]!='&' ) {
		*err = 1;  /* need to start with ampersand */
		c = s[*pi];
	} else *err = 0;

	if ( !*err ) {
		if ( s[*pi+1]=='#' ) c = decode_numeric_entity( s, pi, err );
		else {
			c = decode_html_entity( s, pi, err );
			*unicode = 1;
		}
	}
	if ( *err ) *pi = *pi + 1;

	return c;
}
