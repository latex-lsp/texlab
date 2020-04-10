/*
 * utf8.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <string.h>
#include "utf8.h"

/* UTF-8 encoding

U-00000000 - U-0000007F:  0xxxxxxx 
U-00000080 - U-000007FF:  110xxxxx 10xxxxxx 
U-00000800 - U-0000FFFF:  1110xxxx 10xxxxxx 10xxxxxx 
U-00010000 - U-001FFFFF:  11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
U-00200000 - U-03FFFFFF:  111110xx 10xxxxxx 10xxxxxx 10xxxxxx 10xxxxxx 
U-04000000 - U-7FFFFFFF:  1111110x 10xxxxxx 10xxxxxx 10xxxxxx 10xxxxxx 10xxxxxx

*/

static void
utf8_build( unsigned int value, unsigned char out[6], int in_pos, int out_pos )
{
	unsigned int in_mask, out_mask;
	int byte = 0;
	while ( in_pos < 32 ) {
		in_mask = 1 << ( 31 - in_pos );
		out_mask = 1 << ( 7 - out_pos );
		if ( value & in_mask ) out[byte] |= out_mask;
		in_pos++;
		out_pos++;
		if ( out_pos > 7 ) {
			out_pos=2;
			byte++;
		}
	}
}

/* int utf8( in, out[6] );
 *
 *  in is character code 0x0 -> 0x7FFFFFFF
 *  int is number of characters for output
 *
 */
int
utf8_encode( unsigned int value, unsigned char out[6] )
{
	int i;
	for ( i=1; i<6; ++i ) out[i] = 0x80;  /* 10xxxxxx */
	if ( value < 0x80 ) {
		out[0] = 0x0;   /* 0xxxxxxx */
		utf8_build( value, out, 25, 1 );
		return 1;
	} else if ( value < 0x800 ) {
		out[0] = 0xC0;  /* 110xxxxx */
		utf8_build( value, out, 21, 3 );
		return 2;
	} else if ( value < 0x10000 ) {
		out[0] = 0xE0;  /* 1110xxxx */
		utf8_build( value, out, 16, 4 );
		return 3;
	} else if ( value < 0x200000 ) {
		out[0] = 0xF0;  /* 11110xxx */
		utf8_build( value, out, 11, 5 );
		return 4;
	} else if ( value < 0x4000000 ) {
		out[0] = 0xF8;  /* 111110xx */
		utf8_build( value, out,  6, 6 );
		return 5;
	} else if ( value < (unsigned int ) 0x80000000 ) {
		out[0] = 0xFC;  /* 1111110x */
		utf8_build( value, out,  1, 7 );
		return 6;
	} else {
		/* error, above 2^31 bits encodable by UTF-8 */
		return 0;
	}
}

/* Generate UTF8 character as null-terminated string */
void
utf8_encode_str( unsigned int value, char outstr[7] )
{
	unsigned char encoded[6];
	int i, n;
	n = utf8_encode( value, encoded );
	for ( i=0; i<n; ++i )
		outstr[i] = ( char ) encoded[i];
	outstr[n] = '\0';
}

unsigned int
utf8_decode( const char *s, unsigned int *pi )
{
	unsigned int c;
	int i = *pi;
	/* one digit utf-8 */
	if ((s[i] & 128)== 0 ) {
		c = (unsigned int) s[i];
		i += 1;
	} else if ((s[i] & 224)== 192 ) { /* 110xxxxx & 111xxxxx == 110xxxxx */
		c = (( (unsigned int) s[i] & 31 ) << 6) +
			( (unsigned int) s[i+1] & 63 );
		i += 2;
	} else if ((s[i] & 240)== 224 ) { /* 1110xxxx & 1111xxxx == 1110xxxx */
		c = ( ( (unsigned int) s[i] & 15 ) << 12 ) + 
			( ( (unsigned int) s[i+1] & 63 ) << 6 ) +
		 	( (unsigned int) s[i+2] & 63 );
		i += 3;
	} else if ((s[i] & 248)== 240 ) { /* 11110xxx & 11111xxx == 11110xxx */
		c =  ( ( (unsigned int) s[i] & 7 ) << 18 ) +
			( ( (unsigned int) s[i+1] & 63 ) << 12 ) +
			( ( (unsigned int) s[i+2] & 63 ) << 6 ) +
			( (unsigned int) s[i+3] & 63 );
		i+= 4;
	} else if ((s[i] & 252)== 248 ) { /* 111110xx & 111111xx == 111110xx */
		c = ( ( (unsigned int) s[i] & 3 ) << 24 ) +
			( ( (unsigned int) s[i+1] & 63 ) << 18 ) +
			( ( (unsigned int) s[i+2] & 63 ) << 12 ) +
			( ( (unsigned int) s[i+3] & 63 ) << 6 ) +
			( (unsigned int) s[i+4] & 63 );
		i += 5;
	} else if ((s[i] & 254)== 252 ) { /* 1111110x & 1111111x == 1111110x */
		c = ( ( (unsigned int) s[i] & 1 ) << 30 ) + 
			( ( (unsigned int) s[i+1] & 63 ) << 24 ) +
			( ( (unsigned int) s[i+2] & 63 ) << 18 ) +
			( ( (unsigned int) s[i+3] & 63 ) << 12 ) +
			( ( (unsigned int) s[i+4] & 63 ) << 6 ) +
			( (unsigned int) s[i+5] & 63 );
		i += 6;
	} else {
		c = '?';
		i++;
	}
	*pi = i;
	return c;
}

void
utf8_writebom( FILE *outptr )
{
	int i, nc;
	unsigned char code[6];
	nc = utf8_encode( 0xFEFF, code );
	for ( i=0; i<nc; ++i )
		fprintf(outptr,"%c",code[i]);
}

int
utf8_is_bom( const char *p )
{
	const unsigned char *up = ( unsigned char * ) p;

	/* ...if null-terminated string is too short, we're ok */
	if ( up[0]!=0xEF ) return 0;
	if ( up[1]!=0xBB ) return 0;
	if ( up[2]!=0xBF ) return 0;

	return 1;
}

/* utf8_is_emdash()
 *
 *emdash = 0xE2 (-30) 0x80 (-128) 0x94 (-108)
 */
int
utf8_is_emdash( const char *p )
{
	const char emdash[3] = { -30, -128, -108 };
	if ( strncmp( p, emdash, 3 ) ) return 0;
	return 1;
}

/* utf8_is_endash()
 *
 * endash = 0xE2 (-30) 0x80 (-128) 0x93 (-109)
 */
int
utf8_is_endash( const char *p )
{
	const char endash[3] = { -30, -128, -109 };
	if ( strncmp( p, endash, 3 ) ) return 0;
	return 1;
}

