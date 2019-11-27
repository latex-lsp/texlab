/*
 * xml_getencoding.c
 *
 * Copyright (c) Chris Putnam 2007-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "charsets.h"
#include "str.h"
#include "str_conv.h"
#include "xml.h"
#include "xml_encoding.h"

static int
xml_getencodingr( xml *node )
{
	int n = CHARSET_UNKNOWN, m;
	str *s;
	char *t;

	if ( xml_tag_matches( node, "xml" ) ) {
		s = xml_attribute( node, "encoding" );
		if ( str_has_value( s ) ) {
			t = str_cstr( s );
			if ( !strcasecmp( t, "UTF-8" ) )
				n = CHARSET_UNICODE;
			else if ( !strcasecmp( t, "UTF8" ) )
				n = CHARSET_UNICODE;
			else if ( !strcasecmp( t, "GB18030" ) )
				n = CHARSET_GB18030;
			else n = charset_find( t );
			if ( n==CHARSET_UNKNOWN ) {
				fprintf( stderr, "Warning: did not recognize encoding '%s'\n", t );
			}
		}
	}
        if ( node->down ) {
		m = xml_getencodingr( node->down );
		if ( m!=CHARSET_UNKNOWN ) n = m;
	}
        if ( node->next ) {
		m = xml_getencodingr( node->next );
		if ( m!=CHARSET_UNKNOWN ) n = m;
	}

	return n;
}

int
xml_getencoding( str *s )
{
	int file_charset = CHARSET_UNKNOWN;
	str descriptor;
	xml descriptxml;
	char *p, *q;

	p = strstr( str_cstr( s ), "<?xml" );
	if ( !p ) p = strstr( str_cstr( s ), "<?XML" );
	if ( p ) {
		q = strstr( p, "?>" );
		if ( q ) {
			str_init( &descriptor );
			str_segcpy( &descriptor, p, q+2 );
			xml_init( &descriptxml );
			xml_parse( str_cstr( &descriptor ), &descriptxml );
			file_charset = xml_getencodingr( &descriptxml );
			xml_free( &descriptxml );
			str_free( &descriptor );
			str_segdel( s, p, q+2 );
		}
	}
	return file_charset;
}
