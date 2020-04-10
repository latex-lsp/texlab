/*
 * pages.c
 *
 * Copyright (c) Chris Putnam 2016-2019
 *
 * Program and source code released under GPL verison 2
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "is_ws.h"
#include "utf8.h"
#include "pages.h"

/* extract_range()
 *
 * Handle input strings like:
 *
 * "1-15"
 * " 1 - 15 "
 * " 1000--- 1500"
 * " 1 <<em-dash>> 10"
 * " 107 111"
 */
static void
extract_range( str *input, str *begin, str *end )
{
	/* -30 is the first character of a UTF8 em-dash and en-dash */
	const char terminators[] = { ' ', '-', '\t', '\r', '\n', -30, '\0' };
	const char *p;

	str_empty( begin );
	str_empty( end );

	if ( input->len==0 ) return;

	p = skip_ws( str_cstr( input ) );
	while ( *p && !strchr( terminators, *p ) )
		str_addchar( begin, *p++ );

	p = skip_ws( p );

	while ( *p=='-' ) p++;
	while ( utf8_is_emdash( p ) ) p+=3;
	while ( utf8_is_endash( p ) ) p+=3;

	p = skip_ws( p );

	while ( *p && !strchr( terminators, *p ) )
		str_addchar( end, *p++ );
}

int
pages_add( fields *bibout, char *outtag, str *invalue, int level )
{
	int fstatus, status = 1;
	str start, stop;

	str_init( &start );
	str_init( &stop );

	extract_range( invalue, &start, &stop );

	if ( str_memerr( &start ) || str_memerr( &stop ) ) {
		status = 0;
		goto out;
	}

	if ( start.len>0 ) {
		fstatus = fields_add( bibout, "PAGES:START", str_cstr( &start ), level );
		if ( fstatus!=FIELDS_OK ) {
			status = 0;
			goto out;
		}
	}

	if ( stop.len>0 ) {
		fstatus = fields_add( bibout, "PAGES:STOP", str_cstr( &stop ), level );
		if ( fstatus!=FIELDS_OK ) status = 0;
	}

out:
	str_free( &start );
	str_free( &stop );
	return status;
}

