/*
 * is_ws.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include "is_ws.h"

/* is_ws(), is whitespace */
int 
is_ws( const char ch )
{
	if ( ch==' ' || ch=='\n' || ch=='\t' || ch=='\r' ) return 1;
	else return 0;
}

const char *
skip_ws( const char *p )
{
	if ( p ) {
		while ( is_ws( *p ) ) p++;
	}
	return p;
}

const char *
skip_notws( const char *p )
{
	if ( p ) {
		while ( *p && !is_ws( *p ) ) p++;
	}
	return p;
}

const char *
skip_line( const char *p )
{
	/* ...skip until end-of-line markers */
	while ( *p && *p!='\n' && *p!='\r' ) p++;

	/* ...skip end-of-line marker */
	if ( *p=='\r' ) p++; /* for CR LF or just CR end of lines */
	if ( *p=='\n' ) p++;

	return p;
}
