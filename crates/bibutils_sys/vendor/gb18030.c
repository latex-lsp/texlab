#include <stdio.h>
#include "gb18030.h"

/* GB18030-2000 is an encoding of Unicode character used in China
 *
 * {0x00-0x7f} are one byte characters identical to US-ASCII
 * {0x80} is properly undefined, but many GB18030 encodings make
 *      it the Euro sign (Unicode 0x20AC), so use that
 * {0x81-0xFE}{0x40-0x7E,0x80-0xFE} a full superset of GBK (with fallback 
 *      mappings)
 * {0x81-0xFE}{0x30-0x39}{0x81-0xFE}{0x30-0x39} maps linearly to ISO 10646
 *      GB+81308130 = U+0080 up to U+FFFF
 *      GB+90308130 = U+10000 up to U+10FFFF skipping mappings already
 *                     defined in 1-byte and 2-byte areas.
 *
 * Truth is it's a bit of a mess algorithmically as it doesn't multiply
 * encode characters, so there are holes in the Unicode mapping that
 * should be avoided.
 */

/* This is a "small" region that needs explicit enumeration */
#include "gb18030_enumeration.c"

static int
in_range( unsigned char n, unsigned char low, unsigned char high )
{
	if ( n < low || n > high ) return 0;
	return 1;
}


/* Get GB 18030 from Unicode Value in Table */
static int
gb18030_unicode_table_lookup( unsigned int unicode, unsigned char out[4] )
{
	int i, j;
	if ( unicode >= 0x0080 && unicode <= 0xFFE5 ) {
		/* list is sorted, so should do binary search here */
		for ( i=0; i<ngb18030_enums; ++i ) {
			if ( unicode == gb18030_enums[i].unicode ) {
				for ( j=0; j<gb18030_enums[i].len; ++j )
					out[j] = gb18030_enums[i].bytes[j];
				return gb18030_enums[i].len;
			}
		}
	}
	return 0;
}

static int
gb18030_match( unsigned char *s, const unsigned char *bytes, unsigned char len )
{
	int i;
	for ( i=0; i<len; ++i )
		if ( ( s[i])!=bytes[i] ) return 0;
	return 1;
}

static unsigned int
gb18030_table_lookup( unsigned char *uc, unsigned char len, int *found )
{
	unsigned int i;
	*found = 0;
	for ( i=0; i<ngb18030_enums; ++i ) {
		if ( gb18030_enums[i].len!=len ) continue;
		if ( gb18030_match( &(uc[0]), gb18030_enums[i].bytes, len ) ) {
			*found = 1;
			return gb18030_enums[i].unicode;
		}
	}
	return '?';
}


static int
gb18030_unicode_range_lookup( unsigned int unicode, unsigned char out[4] ) 
{
	return 0;
}

static int
gb18030_range_lookup( unsigned char *s, /* unsigned char len = 4 only */ int *found )
{
	*found = 0;
	return 0;
#if 0
  <!-- Roundtrip-mappings that can be enumerated
       Note that GB 18030 defines roundtrip mappings for all Unicode code points U+0000..U+10ffff.
       This would require 1.1 million <a> elements.
       However, most four-byte GB 18030 mappings can be enumerated efficiently within distinct ranges.
       Therefore, we use <range> elements for all but the 31000 or so assignments above.
    -->
  <range uFirst="0452" uLast="200F"  bFirst="81 30 D3 30" bLast="81 36 A5 31"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="2643" uLast="2E80"  bFirst="81 37 A8 39" bLast="81 38 FD 38"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="361B" uLast="3917"  bFirst="82 30 A6 33" bLast="82 30 F2 37"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="3CE1" uLast="4055"  bFirst="82 31 D4 38" bLast="82 32 AF 32"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="4160" uLast="4336"  bFirst="82 32 C9 37" bLast="82 32 F8 37"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="44D7" uLast="464B"  bFirst="82 33 A3 39" bLast="82 33 C9 31"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="478E" uLast="4946"  bFirst="82 33 E8 38" bLast="82 34 96 38"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="49B8" uLast="4C76"  bFirst="82 34 A1 31" bLast="82 34 E7 33"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="9FA6" uLast="D7FF"  bFirst="82 35 8F 33" bLast="83 36 C7 38"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="E865" uLast="F92B"  bFirst="83 36 D0 30" bLast="84 30 85 34"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="FA2A" uLast="FE2F"  bFirst="84 30 9C 38" bLast="84 31 85 37"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="FFE6" uLast="FFFF"  bFirst="84 31 A2 34" bLast="84 31 A4 39"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
  <range uFirst="10000" uLast="10FFFF"  bFirst="90 30 81 30" bLast="E3 32 9A 35"  bMin="81 30 81 30" bMax="FE 39 FE 39"/>
#endif
}

unsigned int
gb18030_to_unicode( unsigned char *s, unsigned char len )
{
	unsigned int ret;
	int found;
	ret = gb18030_table_lookup( s, len, &found );
	if ( !found && len==4 ) {
		ret = gb18030_range_lookup( s, &found );
		if ( !found ) ret = '?';
	}
	return ret;
}

/*
 * Convert unicode character to gb18030
 *
 * returns number of characters for output
 */
int
gb18030_encode( unsigned int unicode, unsigned char out[4] )
{
	int len;
	if ( unicode < 0x80 ) {
		out[0] = unicode;
		len = 1;
	} else {
		len = gb18030_unicode_table_lookup( unicode, out );
		if ( !len )
			len = gb18030_unicode_range_lookup( unicode, out ); 
	}
	return len;
}

/*
 * Decode a gb18030 character into unicode
 */
unsigned int
gb18030_decode( char *s, unsigned int *pi )
{
	unsigned int c;
	unsigned char uc[4];
	int i = *pi;
	uc[0] = ( unsigned char ) s[i];
	if ( ( uc[0] & 128 ) == 0 ) {
		c = ( unsigned int ) uc[0];
		i += 1;
	} else if ( uc[0] == 0x80 ) {
		c = 0x20AC;
		i += 1;
	} else if ( uc[0] != 0xFF ) { /* multi-byte character */
		uc[1] = ( unsigned char ) s[i+1];
		uc[2] = ( unsigned char ) s[i+2];
		uc[3]= ( unsigned char ) s[i+3];
		if ( in_range( uc[1], 0x40, 0x7e ) || in_range( uc[1], 0x80, 0xfe ) ) {
			/* two-byte character */
			c = gb18030_to_unicode( &(uc[0]), 2 );
			i += 2;
		} else if ( in_range( uc[1], 0x30, 0x39 ) &&
		            in_range( uc[2], 0x81, 0xfe ) &&
		            in_range( uc[3], 0x30, 0x39 ) ) {
			/* four-byte character */
			c = gb18030_to_unicode( &(uc[0]), 4 );
			i += 4;
		} else {
			/* this is an illegal character */
			c = '?';
			i += 1;
		}
	} else { /* s[i]==0xFF */
		/* this is an illegal character */
		c = '?';
		i += 1;
	}
	*pi = i;
	return c;
}
