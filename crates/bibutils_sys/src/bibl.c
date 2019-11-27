/*
 * bibl.c
 *
 * Copyright (c) Chris Putnam 2005-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include "bibl.h"

void
bibl_init( bibl *b )
{
	b->nrefs = b->maxrefs = 0L;
	b->ref = NULL;
}

static int
bibl_malloc( bibl * b )
{
	int alloc = 50;
	b->nrefs = 0;
	b->ref = ( fields ** ) malloc( sizeof( fields* ) * alloc );
	if ( b->ref ) {
		b->maxrefs = alloc;
		return 1;
	} else {
		fprintf( stderr, "%s: allocation error\n", __FUNCTION__ );
		return 0;
	}
}

static int
bibl_realloc( bibl * b )
{
	int alloc = b->maxrefs * 2;
	fields **more;
	more = ( fields ** ) realloc( b->ref, sizeof( fields* ) * alloc );
	if ( more ) {
		b->ref = more;
		b->maxrefs = alloc;
		return 1;
	} else {
		fprintf( stderr, "%s: allocation error\n", __FUNCTION__ );
		return 0;
	}
}

int
bibl_addref( bibl *b, fields *ref )
{
	int ok = 1;
	if ( b->maxrefs==0 ) ok = bibl_malloc( b );
	else if ( b->nrefs >= b->maxrefs ) ok = bibl_realloc( b );
	if ( ok ) {
		b->ref[ b->nrefs ] = ref;
		b->nrefs++;
	}
	return ok;
}

void
bibl_free( bibl *b )
{
	long i;
	for ( i=0; i<b->nrefs; ++i )
		fields_delete( b->ref[i] );
	if ( b->ref ) free( b->ref );
	b->ref = NULL;
	b->nrefs = b->maxrefs = 0;
}

/* bibl_copy()
 *
 * returns 1 on success, 0 on failure (memory error)
 */
int
bibl_copy( bibl *bout, bibl *bin )
{
	fields *refin, *refout;
	int i, j, n, status, ok, level;
	char *tag, *value;
	for ( i=0; i<bin->nrefs; ++i ) {
		refin = bin->ref[i];
		refout = fields_new();
		if ( !refout ) return 0;
		n = fields_num( refin );
		for ( j=0; j<n; ++j ) {
			tag   = fields_tag( refin, j, FIELDS_CHRP );
			value = fields_value( refin, j, FIELDS_CHRP );
			level = fields_level( refin, j );
			if ( tag && value ) {
				status = fields_add_can_dup( refout, tag, value, level );
				if ( status!=FIELDS_OK ) return 0;
			}
		}
		ok = bibl_addref( bout, refout );
		if ( !ok ) return 0;
	}
	return 1;
}

