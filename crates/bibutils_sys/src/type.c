/*
 * type.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "type.h"

static int
is_genre_element( fields *in, int n )
{
	char *tag;

	tag = fields_tag( in, n, FIELDS_CHRP );

	if ( !strcasecmp( tag, "GENRE:MARC"     ) ) return 1;
	if ( !strcasecmp( tag, "GENRE:BIBUTILS" ) ) return 1;
	if ( !strcasecmp( tag, "GENRE:UNKNOWN"  ) ) return 1;

	return 0;
}

static int
is_resource_element( fields *in, int n )
{
	if ( !strcasecmp( fields_tag( in, n, FIELDS_CHRP ), "RESOURCE" ) ) return 1;
	return 0;
}

static int
is_issuance_element( fields *in, int n )
{
	if ( !strcasecmp( fields_tag( in, n, FIELDS_CHRP ), "ISSUANCE" ) ) return 1;
	else return 0;
}

static int
match_hints( const char *value, int level, const char *match_name, int match_level )
{
	if ( strcasecmp( value, match_name ) ) return 0;
	if ( match_level!=LEVEL_ANY && level!=match_level ) return 0;
	return 1;
}

/* type_from_mods_hints()
 *
 * We return the first match from the match list that works...this makes us
 * independent of how the genre hints are internally stored in fields *in.
 *
 * Thus we can distinguish between 'book' and 'book chapter' in which book is
 * at different MODS levels by match_type arrays of:
 *
 * ...
 * { "book",    TYPE_BOOK,          LEVEL_MAIN },
 * { "book",    TYPE_BOOKCHAPTER,   LEVEL_ANY  },
 * ...
 *
 * e.g. "book" at LEVEL_ANY matches any values of "book" not caught by the "book" LEVEL_MAIN line
 *
 */
int
type_from_mods_hints( fields *in, int mode, match_type matches[], int nmatches, int type_unknown )
{
	int i, j, level, type = type_unknown;
	char *value;

	for ( i=0; i<nmatches; ++i ) {

		for ( j=0; j<in->n; ++j ) {
			if ( mode==TYPE_FROM_GENRE    && !is_genre_element( in, j ) )    continue;
			if ( mode==TYPE_FROM_RESOURCE && !is_resource_element( in, j ) ) continue;
			if ( mode==TYPE_FROM_ISSUANCE && !is_issuance_element( in, j ) ) continue;
			value = fields_value( in, j, FIELDS_CHRP );
			level = fields_level( in, j );
			if ( match_hints( value, level, matches[i].name, matches[i].level ) ) {
				if ( type==type_unknown ) type = matches[i].type;
			}
		}
	}

	return type;
}
