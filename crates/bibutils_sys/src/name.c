/*
 * name.c
 *
 * mangle names w/ and w/o commas
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <ctype.h>
#include <string.h>
#include "utf8.h"
#include "unicode.h"
#include "is_ws.h"
#include "str.h"
#include "fields.h"
#include "slist.h"
#include "intlist.h"
#include "name.h"

/* name_build_withcomma()
 *
 * reconstruct parsed names in format: 'family|given|given||suffix'
 * to 'family suffix, given given
 */
void
name_build_withcomma( str *s, const char *p )
{
	const char *suffix, *stopat;
	int nseps = 0, nch;

	str_empty( s );

	suffix = strstr( p, "||" );
	if ( suffix ) stopat = suffix;
	else stopat = strchr( p, '\0' );

	 while ( p != stopat ) {
		nch = 0;
		if ( nseps==1 ) {
			if ( suffix ) {
				str_strcatc( s, " " );
				str_strcatc( s, suffix+2 );
			}
			str_addchar( s, ',' );
		}
		if ( nseps ) str_addchar( s, ' ' );
		while ( p!=stopat && *p!='|' ) {
			str_addchar( s, *p++ );
			nch++;
		}
		if ( p!=stopat && *p=='|' ) p++;
		if ( nseps!=0 && nch==1 ) str_addchar( s, '.' );
		nseps++;
	}
}

/* name_findetal()
 *
 * Returns number of final tokens to be skipped in processing
 * of name lists.
 */
int
name_findetal( slist *tokens )
{
	str *s1, *s2;

	if ( tokens->n==0 ) return 0;

	/* ...check last entry for full 'et al.' or variant */
	s2 = slist_str( tokens, tokens->n - 1 );
	if ( !strcasecmp( s2->data, "et alia" ) ||
	     !strcasecmp( s2->data, "et al." )  ||
	     !strcasecmp( s2->data, "et al.," )  ||
	     !strcasecmp( s2->data, "et al" )   ||
	     !strcasecmp( s2->data, "etalia" )  ||
	     !strcasecmp( s2->data, "etal." ) ||
	     !strcasecmp( s2->data, "etal" ) ) {
		return 1;
	}

	if ( tokens->n==1 ) return 0;

	/* ...check last two entries for full 'et' and 'al.' */
	s1 = slist_str( tokens, tokens->n - 2 );
	if ( !strcasecmp( s1->data, "et" ) ) {
		if ( !strcasecmp( s2->data, "alia" ) ||
		     !strcasecmp( s2->data, "al." )  ||
		     !strcasecmp( s2->data, "al.," )  ||
		     !strcasecmp( s2->data, "al" ) ) {
			return 2;
		}
	}

	return 0;
}

#define WITHCOMMA (1)
#define JUNIOR    (2)
#define SENIOR    (4)
#define THIRD     (8)
#define FOURTH    (16)

typedef struct {
	char *s;
	unsigned short value;
} suffix_value_t;

static int
identify_suffix( char *p )
{
	suffix_value_t suffixes[] = {
		{ "Jr."   ,   JUNIOR              },
		{ "Jr"    ,   JUNIOR              },
		{ "Jr.,"  ,   JUNIOR | WITHCOMMA  },
		{ "Jr,"   ,   JUNIOR | WITHCOMMA  },
		{ "Sr."   ,   SENIOR              },
		{ "Sr"    ,   SENIOR              },
		{ "Sr.,"  ,   SENIOR | WITHCOMMA  },
		{ "Sr,"   ,   SENIOR | WITHCOMMA  },
		{ "III"   ,   THIRD               },
		{ "III,"  ,   THIRD  | WITHCOMMA  },
		{ "IV"    ,   FOURTH              },
		{ "IV,"   ,   FOURTH | WITHCOMMA  },
	};
	int i, nsuffixes = sizeof( suffixes ) / sizeof( suffixes[0] );
	for ( i=0; i<nsuffixes; ++i ) {
		if ( !strcmp( p, suffixes[i].s ) )
			return suffixes[i].value;
	}
	return 0;
}

static int
has_suffix( slist *tokens, int begin, int end, int *suffixpos )
{
	int i, ret;
	str *s;

	/* ...check last element, e.g. "H. F. Author, Sr." */
	s = slist_str( tokens, end - 1 );
	ret = identify_suffix( s->data );
	if ( ret ) {
		*suffixpos = end - 1;
		return ret;
	}

	/* ...try to find one after a comma, e.g. "Author, Sr., H. F." */
	for ( i=begin; i<end-1; ++i ) {
		s = slist_str( tokens, i );
		if ( s->len && s->data[ s->len - 1 ]==',' ) {
			s = slist_str( tokens, i+1 );
			ret = identify_suffix( s->data );
			if ( ret ) {
				*suffixpos = i+1;
				return ret;
			}
		}
	}

	return 0;
}

static int
add_given_split( str *name, str *s )
{
	unsigned int unicode_char;
	unsigned int pos = 0;
	char utf8s[7];
	while ( pos < s->len ) {
		unicode_char = utf8_decode( s->data, &pos );
		if ( is_ws( (char) unicode_char ) ) continue;
		else if ( unicode_char==(unsigned int)'.' ) {
			if ( s->data[pos]=='-' ) {
				str_strcatc( name, ".-" );
				pos += 1;
				unicode_char = utf8_decode( s->data, &pos );
				utf8_encode_str( unicode_char, utf8s );
				str_strcatc( name, utf8s );
				str_addchar( name, '.' );
			}
		} else if ( unicode_char==(unsigned int)'-' ) {
			str_strcatc( name, ".-" );
			unicode_char = utf8_decode( s->data, &pos );
			utf8_encode_str( unicode_char, utf8s );
			str_strcatc( name, utf8s );
			str_addchar( name, '.' );
		} else if ( unicode_char==(unsigned int)',' ) { /* nothing */
		} else {
			str_addchar( name, '|' );
			utf8_encode_str( unicode_char, utf8s );
			str_strcatc( name, utf8s );
		}
	}
	return 1;
}

static unsigned char
token_has_no_upper( slist *tokens, int n )
{
	unsigned short m;
	str *s;
	s = slist_str( tokens, n );
	m = unicode_utf8_classify_str( s );
	if ( m & UNICODE_UPPER ) return 0;
	else return 1;
}

static unsigned char
token_has_upper( slist *tokens, int n )
{
	if ( token_has_no_upper( tokens, n ) ) return 0;
	else return 1;
}

static int
name_multielement_nocomma( intlist *given, intlist *family, slist *tokens, int begin, int end, int suffixpos )
{
	int family_start, family_end;
	int i, n;

	/* ...family name(s) */
	family_start = family_end = end - 1;
	if ( family_start == suffixpos ) family_start = family_end = end - 2;

	/* ...if family name is capitalized, then look for first non-capitalized
	 * ...token and combine range to family name, e.g. single quoted parts of
	 * ..."Ludwig 'von Beethoven'"
	 * ..."Johannes Diderik 'van der Waals'"
	 * ..."Charles Louis Xavier Joseph 'de la Valla Poussin' */
	if ( token_has_upper( tokens, family_start ) ) {
		i = family_start - 1;
		n = -1;
		while ( i >= begin && ( n==-1 || token_has_no_upper( tokens, i ) ) ) {
			if ( token_has_no_upper( tokens, i ) ) n = i;
			i--;
		}
		if ( n != -1 ) family_start = n;
	}
	for ( i=family_start; i<family_end+1; i++ )
		intlist_add( family, i );

	/* ...given names */
	for ( i=begin; i<end-1; i++ ) {
		if ( i>=family_start && i<=family_end ) continue;
		if ( i==suffixpos ) continue;
		intlist_add( given, i );
	}

	return 1;
}

static int
name_multielement_comma( intlist *given, intlist *family, slist *tokens, int begin, int end, int comma, int suffixpos )
{
	str *s;
	int i;

	/* ...family names */
	for ( i=begin; i<comma; ++i ) {
		if ( i==suffixpos ) continue;
		intlist_add( family, i );
	}
	s = slist_str( tokens, comma );
	str_trimend( s, 1 ); /* remove comma */
	intlist_add( family, comma );

	/* ...given names */
	for ( i=comma+1; i<end; ++i ) {
		if ( i==suffixpos ) continue;
		intlist_add( given, i );
	}

	return 1;
}

static int
name_mutlielement_build( str *name, intlist *given, intlist *family, slist *tokens )
{
	unsigned short case_given = 0, case_family = 0, should_split = 0;
	str *s;
	int i, m;

	/* ...copy and analyze family name */
	for ( i=0; i<family->n; ++i ) {
		m = intlist_get( family, i );
		s = slist_str( tokens, m );
		if ( i ) str_addchar( name, ' '  );
		str_strcat( name, s );
		case_family |= unicode_utf8_classify_str( s );
	}

	/* ...check given name case */
	for ( i=0; i<given->n; ++i ) {
		m = intlist_get( given, i );
		s = slist_str( tokens, m );
		case_given |= unicode_utf8_classify_str( s );
	}

	if ( ( ( case_family & UNICODE_MIXEDCASE ) == UNICODE_MIXEDCASE ) &&
	     ( ( case_given  & UNICODE_MIXEDCASE ) == UNICODE_UPPER ) ) {
		should_split = 1;
	}

	for ( i=0; i<given->n; ++i ) {
		m = intlist_get( given, i );
		s = slist_str( tokens, m );
		if ( !should_split ) {
			str_addchar( name, '|' );
			str_strcat( name, s );
		} else add_given_split( name, s );
	}
	return 1;
}

static int
name_construct_multi( str *outname, slist *tokens, int begin, int end )
{
	int i, suffix, suffixpos=-1, comma=-1;
	intlist given, family;
	str *s;

	intlist_init( &family );
	intlist_init( &given );

	str_empty( outname );

	suffix = has_suffix( tokens, begin, end, &suffixpos );

	for ( i=begin; i<end && comma==-1; i++ ) {
		if ( i==suffixpos ) continue;
		s = slist_str( tokens, i );
		if ( s->data[ s->len -1 ] == ',' ) {
			if ( suffix && i==suffixpos-1 && !(suffix&WITHCOMMA) )
				str_trimend( s, 1 );
			else
				comma = i;
		}
	}

	if ( comma != -1 )
		name_multielement_comma( &given, &family, tokens, begin, end, comma, suffixpos );
	else
		name_multielement_nocomma( &given, &family, tokens, begin, end, suffixpos );

	name_mutlielement_build( outname, &given, &family, tokens );

	if ( suffix ) {
		if ( suffix & JUNIOR ) str_strcatc( outname, "||Jr." );
		if ( suffix & SENIOR ) str_strcatc( outname, "||Sr." );
		if ( suffix & THIRD  ) str_strcatc( outname, "||III" );
		if ( suffix & FOURTH ) str_strcatc( outname, "||IV"  );
	}

	intlist_free( &given );
	intlist_free( &family );

	return 1;
}

int
name_addmultielement( fields *info, const char *tag, slist *tokens, int begin, int end, int level )
{
	int status, ok = 1;
	str name;

	str_init( &name );

	name_construct_multi( &name, tokens, begin, end );
	status = fields_add_can_dup( info, tag, name.data, level );
	if ( status!=FIELDS_OK ) ok = 0;

	str_free( &name );

	return ok;
}


/* name_addsingleelement()
 *
 * Treat names that are single tokens, e.g. {Random Corporation, Inc.} in bibtex
 * as a name that should not be mangled (e.g. AUTHOR:ASIS or AUTHOR:CORP, if corp
 * is set).
 */
int
name_addsingleelement( fields *info, const char *tag, const char *name, int level, int corp )
{
	int status, ok = 1;
	str outtag;

	str_init( &outtag );

	str_strcpyc( &outtag, tag );
	if ( !corp ) str_strcatc( &outtag, ":ASIS" );
	else str_strcatc( &outtag, ":CORP" );

	status = fields_add_can_dup( info, outtag.data, name, level );
	if ( status!=FIELDS_OK ) ok = 0;

	str_free( &outtag );
	return ok;
}

/*
 * Takes a single name in a string and parses it.
 * Skipped by bibtex/biblatex that come pre-parsed.
 *
 * Returns 0 on error.
 * Returns 1 on ok.
 * Returns 2 on ok and name in asis list
 * Returns 3 on ok and name in corps list
 */
int
name_parse( str *outname, str *inname, slist *asis, slist *corps )
{
	int status, ret = 1;
	slist tokens;

	str_empty( outname );
	if ( !inname || !inname->len ) return ret;

	slist_init( &tokens );

	if ( asis && slist_find( asis, inname ) !=-1 ) {
		str_strcpy( outname, inname );
		ret = 2;
		goto out;
	} else if ( corps && slist_find( corps, inname ) != -1 ) {
		str_strcpy( outname, inname );
		ret = 3;
		goto out;
	}

	str_findreplace( inname, ",", ", " );
	status = slist_tokenize( &tokens, inname, " ", 1 );
	if ( status!=SLIST_OK ) {
		str_strcpy( outname, inname );
		ret = 2;
		goto out;
	}

	if ( tokens.n==1 ) {
		str_strcpy( outname, inname );
		ret = 2;
	} else {
		name_construct_multi( outname, &tokens, 0, tokens.n );
		ret = 1;
	}

out:

	slist_free( &tokens );

	return ret;
}

static const char *
name_copy( str *name, const char *p )
{
	const char *start, *end, *q;

	str_empty( name );

	start = p = skip_ws( p );

	/* strip tailing whitespace and commas */
	while ( *p && *p!='|' ) p++;

	end = p;
	while ( is_ws( *end ) || *end==',' || *end=='|' || *end=='\0' )
		end--;
	if ( *p=='|' ) p++;

	for ( q=start; q<=end; q++ )
		str_addchar( name, *q );

	return p;
}

/*
 * name_add( info, newtag, data, level )
 *
 * take name(s) in data, multiple names should be separated by
 * '|' characters and divide into individual name, e.g.
 * "H. F. Author|W. G. Author|Q. X. Author"
 *
 * for each name, compare to names in the "as is" or "corporation"
 * lists...these are not personal names and should be added to the
 * bibliography fields directly and should not be mangled
 * 
 * for each personal name, send to appropriate algorithm depending
 * on if the author name is in the format "H. F. Author" or
 * "Author, H. F."
 */
int
name_add( fields *info, const char *tag, const char *q, int level, slist *asis, slist *corps )
{
	int ok, status, nametype, ret = 1;
	str inname, outname;
	slist tokens;

	if ( !q ) return 0;

	slist_init( &tokens );
	strs_init( &inname, &outname, NULL );

	while ( *q ) {

		q = name_copy( &inname, q );

		nametype = name_parse( &outname, &inname, asis, corps );
		if ( !nametype ) { ret = 0; goto out; }

		if ( nametype==1 ) {
			status = fields_add_can_dup( info, tag, outname.data, level );
			ok = ( status==FIELDS_OK ) ? 1 : 0;
		}
		else if ( nametype==2 )
			ok = name_addsingleelement( info, tag, outname.data, level, 0 );
		else
			ok = name_addsingleelement( info, tag, outname.data, level, 1 );

		if ( !ok ) { ret = 0; goto out; }

	}

out:
	strs_free( &inname, &outname, NULL );
	slist_free( &tokens );

	return ret;
}
