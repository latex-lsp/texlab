/*
 * bu_auth.c
 *
 * Copyright (c) Chris Putnam 2017-2019
 *
 * Source code released under the GPL version 2
 */
#include <string.h>
#include "bu_auth.h"

const char *bu_genre[] = {
	"academic journal",
	"airtel",
	"collection",
	"communication",
	"Diploma thesis",
	"Doctoral thesis",
	"electronic",
	"e-mail communication"
	"Habilitation thesis",
	"handwritten note",
	"hearing",
	"journal article",
	"Licentiate thesis",
	"magazine",
	"magazine article",
	"manuscript",
	"Masters thesis",
	"memo",
	"miscellaneous",
	"newspaper article",
	"pamphlet",
	"Ph.D. thesis",
	"press release",
	"teletype",
	"television broadcast",
	"unpublished"
};
int nbu_genre = sizeof( bu_genre ) / sizeof( const char *);

static int
position_in_list( const char *list[], int nlist, const char *query )
{
	int i;
	for ( i=0; i<nlist; ++i ) {
		if ( !strcasecmp( query, list[i] ) ) return i;
	}
	return -1;
}

int
bu_findgenre( const char *query )
{
	return position_in_list( bu_genre, nbu_genre, query );
}

int
is_bu_genre( const char *query )
{
	if ( bu_findgenre( query ) != -1 ) return 1;
	return 0;
}
