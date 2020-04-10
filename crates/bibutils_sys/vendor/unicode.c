/*
 * unicode.c
 *
 * Helper unicode functions/values to determine the
 * types of unicode characters.
 */
#include "utf8.h"
#include "unicode.h"

typedef struct {
	unsigned int value;
	unsigned short info;
} unicodeinfo_t;

static unicodeinfo_t unicodeinfo[] = {
	{  48, UNICODE_NUMBER }, /* 0 */
	{  49, UNICODE_NUMBER }, /* 1 */
	{  50, UNICODE_NUMBER }, /* 2 */
	{  51, UNICODE_NUMBER }, /* 3 */
	{  52, UNICODE_NUMBER }, /* 4 */
	{  53, UNICODE_NUMBER }, /* 5 */
	{  54, UNICODE_NUMBER }, /* 6 */
	{  55, UNICODE_NUMBER }, /* 7 */
	{  56, UNICODE_NUMBER }, /* 8 */
	{  57, UNICODE_NUMBER }, /* 9 */
	{  65, UNICODE_UPPER }, /* Latin Capital A */
	{  66, UNICODE_UPPER }, /* Latin Capital B */
	{  67, UNICODE_UPPER }, /* Latin Capital C */
	{  68, UNICODE_UPPER }, /* Latin Capital D */
	{  69, UNICODE_UPPER }, /* Latin Capital E */
	{  70, UNICODE_UPPER }, /* Latin Capital F */
	{  71, UNICODE_UPPER }, /* Latin Capital G */
	{  72, UNICODE_UPPER }, /* Latin Capital H */
	{  73, UNICODE_UPPER }, /* Latin Capital I */
	{  74, UNICODE_UPPER }, /* Latin Capital J */
	{  75, UNICODE_UPPER }, /* Latin Capital K */
	{  76, UNICODE_UPPER }, /* Latin Capital L */
	{  77, UNICODE_UPPER }, /* Latin Capital M */
	{  78, UNICODE_UPPER }, /* Latin Capital N */
	{  79, UNICODE_UPPER }, /* Latin Capital O */
	{  80, UNICODE_UPPER }, /* Latin Capital P */
	{  81, UNICODE_UPPER }, /* Latin Capital Q */
	{  82, UNICODE_UPPER }, /* Latin Capital R */
	{  83, UNICODE_UPPER }, /* Latin Capital S */
	{  84, UNICODE_UPPER }, /* Latin Capital T */
	{  85, UNICODE_UPPER }, /* Latin Capital U */
	{  86, UNICODE_UPPER }, /* Latin Capital V */
	{  87, UNICODE_UPPER }, /* Latin Capital W */
	{  88, UNICODE_UPPER }, /* Latin Capital X */
	{  89, UNICODE_UPPER }, /* Latin Capital Y */
	{  90, UNICODE_UPPER }, /* Latin Capital Z */
	{  97, UNICODE_LOWER }, /* Latin Small   a */
	{  98, UNICODE_LOWER }, /* Latin Small   b */
	{  99, UNICODE_LOWER }, /* Latin Small   c */
	{ 100, UNICODE_LOWER }, /* Latin Small   d */
	{ 101, UNICODE_LOWER }, /* Latin Small   e */
	{ 102, UNICODE_LOWER }, /* Latin Small   f */
	{ 103, UNICODE_LOWER }, /* Latin Small   g */
	{ 104, UNICODE_LOWER }, /* Latin Small   h */
	{ 105, UNICODE_LOWER }, /* Latin Small   i */
	{ 106, UNICODE_LOWER }, /* Latin Small   j */
	{ 107, UNICODE_LOWER }, /* Latin Small   k */
	{ 108, UNICODE_LOWER }, /* Latin Small   l */
	{ 109, UNICODE_LOWER }, /* Latin Small   m */
	{ 110, UNICODE_LOWER }, /* Latin Small   n */
	{ 111, UNICODE_LOWER }, /* Latin Small   o */
	{ 112, UNICODE_LOWER }, /* Latin Small   p */
	{ 113, UNICODE_LOWER }, /* Latin Small   q */
	{ 114, UNICODE_LOWER }, /* Latin Small   r */
	{ 115, UNICODE_LOWER }, /* Latin Small   s */
	{ 116, UNICODE_LOWER }, /* Latin Small   t */
	{ 117, UNICODE_LOWER }, /* Latin Small   u */
	{ 118, UNICODE_LOWER }, /* Latin Small   v */
	{ 119, UNICODE_LOWER }, /* Latin Small   w */
	{ 120, UNICODE_LOWER }, /* Latin Small   x */
	{ 121, UNICODE_LOWER }, /* Latin Small   y */
	{ 122, UNICODE_LOWER }, /* Latin Small   z */
	{ 192, UNICODE_UPPER }, /* Latin Capital A with grave */
	{ 193, UNICODE_UPPER }, /* Latin Capital A with acute */
	{ 194, UNICODE_UPPER }, /* Latin Capital A with circumflex */
	{ 195, UNICODE_UPPER }, /* Latin Capital A with tilde */
	{ 196, UNICODE_UPPER }, /* Latin Capital A with diuresis */
	{ 197, UNICODE_UPPER }, /* Latin Capital A with ring above */
	{ 198, UNICODE_UPPER }, /* Latin Capital AE */
	{ 199, UNICODE_UPPER }, /* Latin Capital C with cedilla */
	{ 200, UNICODE_UPPER }, /* Latin Capital E with grave */
	{ 201, UNICODE_UPPER }, /* Latin Capital E with acute */
	{ 202, UNICODE_UPPER }, /* Latin Capital E with circumflex */
	{ 203, UNICODE_UPPER }, /* Latin Capital E with diuresis */
	{ 204, UNICODE_UPPER }, /* Latin Capital I with grave */
	{ 205, UNICODE_UPPER }, /* Latin Capital I with acute */
	{ 206, UNICODE_UPPER }, /* Latin Capital I with circumflex */
	{ 207, UNICODE_UPPER }, /* Latin Capital I with diuresis */
	{ 208, UNICODE_UPPER }, /* Latin Capital ETH */
	{ 209, UNICODE_UPPER }, /* Latin Capital N with tilde */
	{ 210, UNICODE_UPPER }, /* Latin Capital O with grave */
	{ 211, UNICODE_UPPER }, /* Latin Capital O with acute */
	{ 212, UNICODE_UPPER }, /* Latin Capital O with circumflex */
	{ 213, UNICODE_UPPER }, /* Latin Capital O with tilde */
	{ 214, UNICODE_UPPER }, /* Latin Captial O with diaeresis */
	{ 216, UNICODE_UPPER }, /* Latin Capital O with stroke */
	{ 217, UNICODE_UPPER }, /* Latin Capital U with grave */
	{ 218, UNICODE_UPPER }, /* Latin Capital U with acute */
	{ 219, UNICODE_UPPER }, /* Latin Capital U with circumflex */
	{ 220, UNICODE_UPPER }, /* Latin Capital U with diaeresis */
	{ 221, UNICODE_UPPER }, /* Latin Capital Y with acute */
	{ 222, UNICODE_UPPER }, /* Latin Capital THORN */
	{ 223, UNICODE_LOWER }, /* German sz ligature */
	{ 224, UNICODE_LOWER }, /* Latin Small   a with grave */
	{ 225, UNICODE_LOWER }, /* Latin Small   a with acute */
	{ 226, UNICODE_LOWER }, /* Latin Small   a with circumflex */
	{ 227, UNICODE_LOWER }, /* Latin Small   a with tilde */
	{ 228, UNICODE_LOWER }, /* Latin Small   a with diuresis */
	{ 229, UNICODE_LOWER }, /* Latin Small   a with ring above */
	{ 230, UNICODE_LOWER }, /* Latin Small   ae */
	{ 231, UNICODE_LOWER }, /* Latin Small   c with cedilla */
	{ 232, UNICODE_LOWER }, /* Latin Small   e with grave */
	{ 233, UNICODE_LOWER }, /* Latin Small   e with acute */
	{ 234, UNICODE_LOWER }, /* Latin Small   e with circumflex */
	{ 235, UNICODE_LOWER }, /* Latin Small   e with diuresis */
	{ 236, UNICODE_LOWER }, /* Latin Small   i with grave */
	{ 237, UNICODE_LOWER }, /* Latin Small   i with acute */
	{ 238, UNICODE_LOWER }, /* Latin Small   i with circumflex */
	{ 239, UNICODE_LOWER }, /* Latin Small   i with diuresis */
	{ 240, UNICODE_LOWER }, /* Latin Small   eth */
	{ 241, UNICODE_LOWER }, /* Latin Small   n with tilde */
	{ 242, UNICODE_LOWER }, /* Latin Small   o with grave */
	{ 243, UNICODE_LOWER }, /* Latin Small   o with acute */
	{ 244, UNICODE_LOWER }, /* Latin Small   o with circumflex */
	{ 245, UNICODE_LOWER }, /* Latin Small   o with tilde */
	{ 246, UNICODE_LOWER }, /* Latin Small   o with diaeresis */
	{ 248, UNICODE_LOWER }, /* Latin Small   o with stroke */
	{ 249, UNICODE_LOWER }, /* Latin Small   u with grave */
	{ 250, UNICODE_LOWER }, /* Latin Small   u with acute */
	{ 251, UNICODE_LOWER }, /* Latin Small   u with circumflex */
	{ 252, UNICODE_LOWER }, /* Latin Small   u with diaeresis */
	{ 253, UNICODE_LOWER }, /* Latin Small   y with acute */
	{ 254, UNICODE_LOWER }, /* Latin Small   thorn */
	{ 255, UNICODE_LOWER }, /* Latin Small   y with diaeresis */
	{ 256, UNICODE_UPPER }, /* Latin Capital A with macron */
	{ 257, UNICODE_LOWER }, /* Latin Small   a with macron */
	{ 258, UNICODE_UPPER }, /* Latin Capital A with breve */
	{ 259, UNICODE_LOWER }, /* Latin Small   a with breve */
	{ 260, UNICODE_UPPER }, /* Latin Capital A with ogonek */
	{ 261, UNICODE_LOWER }, /* Latin Small   a with ogonek */
	{ 262, UNICODE_UPPER }, /* Latin Capital C with acute */
	{ 263, UNICODE_LOWER }, /* Latin Small   c with acute */
	{ 264, UNICODE_UPPER }, /* Latin Capital C with circumflex */
	{ 265, UNICODE_LOWER }, /* Latin Small   c with circumflex */
	{ 266, UNICODE_UPPER }, /* Latin Capital C with dot above */
	{ 267, UNICODE_LOWER }, /* Latin Small   c with dot above */
	{ 268, UNICODE_UPPER }, /* Latin Capital C with caron (hacek) */
	{ 269, UNICODE_LOWER }, /* Latin Small   c with caron (hacek) */
	{ 270, UNICODE_UPPER }, /* Latin Capital D with caron (hacek) */
	{ 271, UNICODE_LOWER }, /* Latin Small   d with caron (hacek) */
	{ 272, UNICODE_UPPER }, /* Latin Capital D with stroke */
	{ 273, UNICODE_LOWER }, /* Latin Small   d with stroke */
	{ 274, UNICODE_UPPER }, /* Latin Capital E with macron */
	{ 275, UNICODE_LOWER }, /* Latin Small   e with macron */
	{ 276, UNICODE_UPPER }, /* Latin Capital E with breve */
	{ 277, UNICODE_LOWER }, /* Latin Small   e with breve */
	{ 278, UNICODE_UPPER }, /* Latin Capital E with dot above */
	{ 279, UNICODE_LOWER }, /* Latin Small   e with dot above */
	{ 280, UNICODE_UPPER }, /* Latin Capital E with ogonek */
	{ 281, UNICODE_LOWER }, /* Latin Small   e with ogonek */
	{ 282, UNICODE_UPPER }, /* Latin Capital E with caron (hacek) */
	{ 283, UNICODE_LOWER }, /* Latin Small   e with caron */
	{ 284, UNICODE_UPPER }, /* Latin Capital G with circumflex */
	{ 285, UNICODE_LOWER }, /* Latin Small   g with circumflex */
	{ 286, UNICODE_UPPER }, /* Latin Capital G with breve */
	{ 287, UNICODE_LOWER }, /* Latin Small   g with breve */
	{ 288, UNICODE_UPPER }, /* Latin Capital G with dot above */
	{ 289, UNICODE_LOWER }, /* Latin Small   g with dot above */
	{ 290, UNICODE_UPPER }, /* Latin Capital G with cedilla */
	{ 291, UNICODE_LOWER }, /* Latin Small   g with cedilla */
	{ 292, UNICODE_UPPER }, /* Latin Capital H with circumflex */
	{ 293, UNICODE_LOWER }, /* Latin Small   h with circumflex */
	{ 294, UNICODE_UPPER }, /* Latin Capital H with stroke */
	{ 295, UNICODE_LOWER }, /* Latin Small   h with stroke */
	{ 296, UNICODE_UPPER }, /* Latin Capital I with tilde */
	{ 297, UNICODE_LOWER }, /* Latin Small   i with tilde */
	{ 298, UNICODE_UPPER }, /* Latin Capital I with macron */
	{ 299, UNICODE_LOWER }, /* Latin Small   i with macron */
	{ 300, UNICODE_UPPER }, /* Latin Capital I with breve */
	{ 301, UNICODE_LOWER }, /* Latin Small   i with breve */
	{ 302, UNICODE_UPPER }, /* Latin Capital I with ogonek */
	{ 303, UNICODE_LOWER }, /* Latin Small   i with ogonek */
	{ 304, UNICODE_UPPER }, /* Latin Capital I with dot above */
	{ 305, UNICODE_LOWER }, /* Latin Small   i without dot above */
	{ 306, UNICODE_UPPER }, /* Latin Capital IJ */
	{ 307, UNICODE_LOWER }, /* Latin Small IJ */
	{ 308, UNICODE_UPPER }, /* Latin Capital J with circumflex */
	{ 309, UNICODE_LOWER }, /* Latin Small   j with circumflex */
	{ 310, UNICODE_UPPER }, /* Latin Capital K with cedilla */
	{ 311, UNICODE_LOWER }, /* Latin Small   j with cedilla */
	{ 312, UNICODE_LOWER }, /* Latin Small   kra */
	{ 313, UNICODE_UPPER }, /* Latin Capital L with acute */
	{ 314, UNICODE_LOWER }, /* Latin Small   l with acute */
	{ 315, UNICODE_UPPER }, /* Latin Capital L with cedilla */
	{ 316, UNICODE_LOWER }, /* Latin Small   l with cedilla */
	{ 317, UNICODE_UPPER }, /* Latin Capital L with caron */
	{ 318, UNICODE_LOWER }, /* Latin Small   l with caron */
	{ 319, UNICODE_UPPER }, /* Latin Capital L with middle dot */
	{ 320, UNICODE_LOWER }, /* Latin Small   l with middle dot */
	{ 321, UNICODE_UPPER }, /* Latin Capital L with stroke */
	{ 322, UNICODE_LOWER }, /* Latin Small   l with stroke */
	{ 323, UNICODE_UPPER }, /* Latin Capital N with acute */
	{ 324, UNICODE_LOWER }, /* Latin Small   n with acute */
	{ 325, UNICODE_UPPER }, /* Latin Capital N with cedilla */
	{ 326, UNICODE_LOWER }, /* Latin Small   n with cedilla */
	{ 327, UNICODE_UPPER }, /* Latin Capital N with caron */
	{ 328, UNICODE_LOWER }, /* Latin Small   n with caron */
	{ 329, UNICODE_LOWER }, /* Latin Small   n preceeded by apostrophe */
	{ 330, UNICODE_UPPER }, /* Latin Capital Eng */
	{ 331, UNICODE_LOWER }, /* Latin Small   eng */
	{ 332, UNICODE_UPPER }, /* Latin Capital O with macron */
	{ 333, UNICODE_LOWER }, /* Latin Small   o with macron */
	{ 334, UNICODE_UPPER }, /* Latin Capital O with breve */
	{ 335, UNICODE_LOWER }, /* Latin Small   o with breve */
	{ 336, UNICODE_UPPER }, /* Latin Capital O with double acute */
	{ 337, UNICODE_LOWER }, /* Latin Small   o with double acute */
	{ 338, UNICODE_UPPER }, /* Latin Capital OE */
	{ 339, UNICODE_LOWER }, /* Latin Small   oe */
	{ 340, UNICODE_UPPER }, /* Latin Capital R with acute */
	{ 341, UNICODE_LOWER }, /* Latin Small   r with acute */
	{ 342, UNICODE_UPPER }, /* Latin Capital R with cedilla */
	{ 343, UNICODE_LOWER }, /* Latin Small   r with cedilla */
	{ 344, UNICODE_UPPER }, /* Latin Capital R with caron */
	{ 345, UNICODE_LOWER }, /* Latin Small   r with caron */
	{ 346, UNICODE_UPPER }, /* Latin Capital S with acute */
	{ 347, UNICODE_LOWER }, /* Latin Small   s with acute */
	{ 348, UNICODE_UPPER }, /* Latin Capital S with circumflex */
	{ 349, UNICODE_LOWER }, /* Latin Small   s with circumflex */
	{ 350, UNICODE_UPPER }, /* Latin Capital S with cedilla */
	{ 351, UNICODE_LOWER }, /* Latin Small   s with cedilla */
	{ 352, UNICODE_UPPER }, /* Latin Capital S with caron */
	{ 353, UNICODE_LOWER }, /* Latin Small   s with caron */
	{ 354, UNICODE_UPPER }, /* Latin Capital T with cedilla */
	{ 355, UNICODE_LOWER }, /* Latin Small   t with cedilla */
	{ 356, UNICODE_UPPER }, /* Latin Capital T with caron */
	{ 357, UNICODE_LOWER }, /* Latin Small   t with caron */
	{ 358, UNICODE_UPPER }, /* Latin Capital T with stroke */
	{ 359, UNICODE_LOWER }, /* Latin Small   t with stroke */
	{ 360, UNICODE_UPPER }, /* Latin Capital U with tilde */
	{ 361, UNICODE_LOWER }, /* Latin Small   u with tilde */
	{ 362, UNICODE_UPPER }, /* Latin Capital U with macron */
	{ 363, UNICODE_LOWER }, /* Latin Small   u with macron */
	{ 364, UNICODE_UPPER }, /* Latin Capital U with breve */
	{ 365, UNICODE_LOWER }, /* Latin Small   u with breve */
	{ 366, UNICODE_UPPER }, /* Latin Capital U with ring above */
	{ 367, UNICODE_LOWER }, /* Latin Small   u with ring above */
	{ 368, UNICODE_UPPER }, /* Latin Capital U with double acute */
	{ 369, UNICODE_LOWER }, /* Latin Small   u with double acute */
	{ 370, UNICODE_UPPER }, /* Latin Capital U with ogonek */
	{ 371, UNICODE_LOWER }, /* Latin Small   u with ogonek */
	{ 372, UNICODE_UPPER }, /* Latin Capital W with circumflex */
	{ 373, UNICODE_LOWER }, /* Latin Small   w with circumflex */
	{ 374, UNICODE_UPPER }, /* Latin Capital Y with circumflex */
	{ 375, UNICODE_LOWER }, /* Latin Small   y with circumflex */
	{ 376, UNICODE_UPPER }, /* Latin Capital Y with diaeresis */
	{ 377, UNICODE_UPPER }, /* Latin Capital Z with acute */
	{ 378, UNICODE_LOWER }, /* Latin Small   z with acute */
	{ 379, UNICODE_UPPER }, /* Latin Capital Z with dot above */
	{ 380, UNICODE_LOWER }, /* Latin Small   z with dot above */
	{ 381, UNICODE_UPPER }, /* Latin Capital Z with caron */
	{ 382, UNICODE_LOWER }, /* Latin Small   z with caron */
	{ 383, UNICODE_LOWER }, /* Latin Small   long S */

	{ 461, UNICODE_UPPER }, /* Latin Capital A with caron (hacek) */
	{ 462, UNICODE_LOWER }, /* Latin Small   a with caron (hacek) */
	{ 463, UNICODE_UPPER }, /* Latin Capital I with caron (hacek) */
	{ 464, UNICODE_LOWER }, /* Latin Small   i with caron (hacek) */
	{ 465, UNICODE_UPPER }, /* Latin Capital O with caron (hacek) */
	{ 466, UNICODE_LOWER }, /* Latin Small   o with caron (hacek) */
	{ 467, UNICODE_UPPER }, /* Latin Capital U with caron (hacek) */
	{ 468, UNICODE_LOWER }, /* Latin Small   u with caron (hacek) */

	{ 486, UNICODE_UPPER }, /* Latin Capital G with caron */
	{ 487, UNICODE_LOWER }, /* Latin Small   g with caron */
	{ 488, UNICODE_UPPER }, /* Latin Capital J with caron */
	{ 489, UNICODE_LOWER }, /* Latin Small   j with caron */
	{ 490, UNICODE_UPPER }, /* Latin Capital O with caron */
	{ 491, UNICODE_LOWER }, /* Latin Small   o with caron */

	{ 500, UNICODE_UPPER }, /* Latin Capital G with acute */
	{ 501, UNICODE_LOWER }, /* Latin Small   g with caron */
};

static int nunicodeinfo = sizeof( unicodeinfo ) / sizeof( unicodeinfo[0] );

static int
unicode_find( unsigned int unicode_character )
{
	int min = 0, max = nunicodeinfo, mid;
	while ( min < max ) {
		mid = ( min + max ) / 2;
		if ( unicodeinfo[mid].value < unicode_character )
			min = mid + 1;
		else
			max = mid;
	}
	if ( ( max==min ) && ( unicodeinfo[min].value == unicode_character ) )
		return min;
	else
		return -1;
}

unsigned short
unicode_utf8_classify( char *p )
{
	unsigned int unicode_character, pos = 0;
	int n;
	unicode_character = utf8_decode( p, &pos );
	n = unicode_find( unicode_character );
	if ( n==-1 ) return UNICODE_SYMBOL;
	else return unicodeinfo[n].info;
}

unsigned short
unicode_utf8_classify_str( str *s )
{
	unsigned int unicode_character, pos = 0;
	unsigned short value = 0;
	int n;
	while ( pos < s->len ) {
		unicode_character = utf8_decode( str_cstr( s ), &pos );
		n = unicode_find( unicode_character );
		if ( n==-1 ) value |= UNICODE_SYMBOL;
		else value |= unicodeinfo[n].info;
	}
	return value;
}

