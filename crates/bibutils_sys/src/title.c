/*
 * title.c
 *
 * process titles into title/subtitle pairs for MODS
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "str.h"
#include "fields.h"
#include "title.h"
#include "is_ws.h"

int
title_process( fields *info, const char *tag, const char *value, int level, unsigned char nosplittitle )
{
	str title, subtitle;
	const char *p, *q;
	int status;

	str_init( &title );
	str_init( &subtitle );

	if ( nosplittitle ) q = NULL;
	else {
		q = strstr( value, ": " );
		if ( !q ) q = strstr( value, "? " );
	}

	if ( !q ) str_strcpyc( &title, value );
	else {
		p = value;
		while ( p!=q ) str_addchar( &title, *p++ );
		if ( *q=='?' ) str_addchar( &title, '?' );
		q++;
		q = skip_ws( q );
		while ( *q ) str_addchar( &subtitle, *q++ );
	}

	if ( strncasecmp( "SHORT", tag, 5 ) ) {
		if ( str_has_value( &title ) ) {
			status = fields_add( info, "TITLE", str_cstr( &title ), level );
			if ( status!=FIELDS_OK ) return 0;
		}
		if ( str_has_value( &subtitle ) ) {
			status = fields_add( info, "SUBTITLE", str_cstr( &subtitle ), level );
			if ( status!=FIELDS_OK ) return 0;
		}
	} else {
		if ( str_has_value( &title ) ) {
			status = fields_add( info, "SHORTTITLE", str_cstr( &title ), level );
			if ( status!=FIELDS_OK ) return 0;
		}
		/* no SHORT-SUBTITLE! */
	}

	str_free( &subtitle );
	str_free( &title );

	return 1;
}

/* title_combine()
 *
 * Combine a main title and a subtitle into a full title.
 *
 * Example:
 * 	Main title = "A Clearing in the Distance"
 * 	Subtitle   = "The Biography of Frederick Law Olmstead"
 * 	Full title = "A Clearing in the Distance: The Biography of Frederick Law Olmstead"
 * Example:
 *	Main title = "What Makes a Good Team Player?"
 *	Subtitle   = "Personality and Team Effectiveness"
 *	Full title = "What Makes a Good Team Player? Personality and Team Effectiveness"
 */
void
title_combine( str *fullttl, str *mainttl, str *subttl )
{
	str_empty( fullttl );

	if ( !mainttl ) return;

	str_strcpy( fullttl, mainttl );

	if ( subttl ) {
		if ( str_has_value( mainttl ) ) {
			if ( mainttl->data[ mainttl->len - 1 ] != '?' && mainttl->data[ mainttl->len - 1] != ':' )
				str_strcatc( fullttl, ": " );
			else
				str_strcatc( fullttl, " " );
		}
		str_strcat( fullttl, subttl );
	}
}
