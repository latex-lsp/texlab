/*
 * fields.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include "fields.h"

fields*
fields_new( void )
{
	fields *f = ( fields * ) malloc( sizeof( fields ) );
	if ( f ) fields_init( f );
	return f;
}

void
fields_init( fields *f )
{
	f->used  = NULL;
	f->level = NULL;
	f->tag   = NULL;
	f->data  = NULL;
	f->max   = f->n = 0;
}

void
fields_free( fields *f )
{
	int i;

	for ( i=0; i<f->max; ++i ) {
		str_free( &(f->tag[i]) );
		str_free( &(f->data[i]) );
	}
	if ( f->tag )   free( f->tag );
	if ( f->data )  free( f->data );
	if ( f->used )  free( f->used );
	if ( f->level ) free( f->level );

	fields_init( f );
}

void
fields_delete( fields *f )
{
	fields_free( f );
	free( f );
}

static int
fields_alloc( fields *f )
{
	int i, alloc = 20;

	f->tag   = (str *) malloc( sizeof(str) * alloc );
	f->data  = (str *) malloc( sizeof(str) * alloc );
	f->used  = (int *)    calloc( alloc, sizeof(int) );
	f->level = (int *)    calloc( alloc, sizeof(int) );
	if ( !f->tag || !f->data || !f->used || !f->level ){
		if ( f->tag )   free( f->tag );
		if ( f->data )  free( f->data );
		if ( f->used )  free( f->used );
		if ( f->level ) free( f->level );
		fields_init( f );
		return FIELDS_ERR;
	}

	f->max = alloc;
	f->n = 0;
	for ( i=0; i<alloc; ++i ) {
		str_init( &(f->tag[i]) );
		str_init( &(f->data[i]) );
	}
	return FIELDS_OK;
}

static int
fields_realloc( fields *f )
{
	str *newtags, *newdata;
	int *newused, *newlevel;
	int i, alloc = f->max * 2;

	newtags = (str*) realloc( f->tag, sizeof(str) * alloc );
	newdata = (str*) realloc( f->data, sizeof(str) * alloc );
	newused = (int*) realloc( f->used, sizeof(int) * alloc );
	newlevel= (int*) realloc( f->level, sizeof(int) * alloc );

	if ( newtags )  f->tag   = newtags;
	if ( newdata )  f->data  = newdata;
	if ( newused )  f->used  = newused;
	if ( newlevel ) f->level = newlevel;
	
	if ( !newtags || !newdata || !newused || !newlevel )
		return FIELDS_ERR;

	f->max = alloc;

	for ( i=f->n; i<alloc; ++i ) {
		str_init( &(f->tag[i]) );
		str_init( &(f->data[i]) );
	}

	return FIELDS_OK;
}

int
_fields_add( fields *f, const char *tag, const char *data, int level, int mode )
{
	int i, n, status;
	str *t, *d;

	if ( !tag || !data ) return FIELDS_OK;

	if ( f->max==0 ) {
		status = fields_alloc( f );
		if ( status!=FIELDS_OK ) return status;
	} else if ( f->n >= f->max ) {
		status = fields_realloc( f );
		if ( status!=FIELDS_OK ) return status;
	}

	/* Don't duplicate identical entries if FIELDS_NO_DUPS */
	if ( mode == FIELDS_NO_DUPS ) {
		for ( i=0; i<f->n; i++ ) {
			t = &(f->tag[i]);
			d = &(f->data[i]);
			if ( f->level[i]==level &&
			     !strcasecmp( str_cstr( t ), tag ) &&
			     !strcasecmp( str_cstr( d ), data ) )
				return FIELDS_OK;
		}
	}

	n = f->n;
	f->used[ n ]  = 0;
	f->level[ n ] = level;
	str_strcpyc( &(f->tag[n]), tag );
	str_strcpyc( &(f->data[n]), data );

	if ( str_memerr( &(f->tag[n]) ) || str_memerr( &(f->data[n] ) ) )
		return FIELDS_ERR;

	f->n++;

	return FIELDS_OK;
}

int
_fields_add_tagsuffix( fields *f, const char *tag, const char *suffix,
		const char *data, int level, int mode )
{
	str newtag;
	int ret;

	str_init( &newtag );
	str_mergestrs( &newtag, tag, suffix, NULL );
	if ( str_memerr( &newtag ) ) ret = FIELDS_ERR;
	else ret = _fields_add( f, newtag.data, data, level, mode );
	str_free( &newtag );

	return ret;
}

/* fields_match_level()
 *
 * returns 1 if level matched, 0 if not
 *
 * level==LEVEL_ANY is a special flag meaning any level can match
 */
int
fields_match_level( fields *f, int n, int level )
{
	if ( level==LEVEL_ANY ) return 1;
	if ( fields_level( f, n )==level ) return 1;
	return 0;
}

/* fields_match_tag()
 *
 * returns 1 if tag matches, 0 if not
 *
 */
int
fields_match_tag( fields *info, int n, const char *tag )
{
	if ( !strcmp( fields_tag( info, n, FIELDS_CHRP ), tag ) ) return 1;
	return 0;
}

int
fields_match_casetag( fields *info, int n, const char *tag )
{
	if ( !strcasecmp( fields_tag( info, n, FIELDS_CHRP ), tag ) ) return 1;
	return 0;
}

int
fields_match_tag_level( fields *info, int n, const char *tag, int level )
{
	if ( !fields_match_level( info, n, level ) ) return 0;
	return fields_match_tag( info, n, tag );
}

int
fields_match_casetag_level( fields *info, int n, const char *tag, int level )
{
	if ( !fields_match_level( info, n, level ) ) return 0;
	return fields_match_casetag( info, n, tag );
}

/* fields_find()
 *
 * Return position [0,f->n) for first match of the tag.
 * Return FIELDS_NOTFOUND if tag isn't found.
 */
int
fields_find( fields *f, const char *tag, int level )
{
	int i;

	for ( i=0; i<f->n; ++i ) {
		if ( !fields_match_casetag_level( f, i, tag, level ) )
			continue;
		if ( f->data[i].len ) return i;
		else {
			/* if there is no data for the tag, don't "find" it */
			/* and set "used" so noise is suppressed */
			f->used[i] = 1;
		}
	}

	return FIELDS_NOTFOUND;
}

int
fields_maxlevel( fields *f )
{
	int i, max = 0;

	if ( f->n ) {
		max = f->level[0];
		for ( i=1; i<f->n; ++i ) {
			if ( f->level[i] > max )
				max = f->level[i];
		}
	}

	return max;
}

void
fields_clearused( fields *f )
{
	int i;

	for ( i=0; i<f->n; ++i )
		f->used[i] = 0;
}

void
fields_setused( fields *f, int n )
{
	if ( n >= 0 && n < f->n )
		f->used[n] = 1;
}

/* fields_replace_or_add()
 *
 * return FIELDS_OK on success, FIELDS_ERR on memory error
 */
int
fields_replace_or_add( fields *f, const char *tag, const char *data, int level )
{
	int n = fields_find( f, tag, level );
	if ( n==FIELDS_NOTFOUND ) return fields_add( f, tag, data, level );
	else {
		str_strcpyc( &(f->data[n]), data );
		if ( str_memerr( &(f->data[n]) ) ) return FIELDS_ERR;
		return FIELDS_OK;
	}
}

char *fields_null_value = "\0";

int
fields_used( fields *f, int n )
{
	if ( n >= 0 && n < f->n ) return f->used[n];
	else return 0;
}

int
fields_notag( fields *f, int n )
{
	str *t;
	if ( n >= 0 && n < f->n ) {
		t = &( f->tag[n] );
		if ( t->len > 0 ) return 0;
	}
	return 1;
}

int
fields_nodata( fields *f, int n )
{
	str *d;
	if ( n >= 0 && n < f->n ) {
		d = &( f->data[n] );
		if ( d->len > 0 ) return 0;
	}
	return 1;
}

int
fields_num( fields *f )
{
	return f->n;
}

/*
 * #define FIELDS_CHRP       
 * #define FIELDS_STRP       
 * #define FIELDS_CHRP_NOLEN
 * #define FIELDS_STRP_NOLEN
 * 
 * If the length of the tagged value is zero and the mode is
 * FIELDS_STRP_NOLEN or FIELDS_CHRP_NOLEN, return a pointer to
 * a static null string as the data field could be new due to
 * the way str handles initialized strings with no data.
 *
 */

void *
fields_value( fields *f, int n, int mode )
{
	intptr_t retn;

	if ( n<0 || n>= f->n ) return NULL;

	if ( mode & FIELDS_SETUSE_FLAG )
		fields_setused( f, n );

	if ( mode & FIELDS_STRP_FLAG )
		return &(f->data[n]);
	else if ( mode & FIELDS_POSP_FLAG ) {
		retn = n;
		return ( void * ) retn; /* Rather pointless */
	} else {
		if ( f->data[n].len )
			return f->data[n].data;
		else
			return fields_null_value;
	}
}

void *
fields_tag( fields *f, int n, int mode )
{
	intptr_t retn;

	if ( n<0 || n>= f->n ) return NULL;

	if ( mode & FIELDS_STRP_FLAG )
		return &(f->tag[n]);
	else if ( mode & FIELDS_POSP_FLAG ) {
		retn = n;
		return ( void * ) retn; /* Rather pointless */
	} else {
		if ( f->tag[n].len )
			return f->tag[n].data;
		else
			return fields_null_value;
	}
}

int
fields_level( fields *f, int n )
{
	if ( n<0 || n>= f->n ) return 0;
	return f->level[n];
}

void *
fields_findv( fields *f, int level, int mode, const char *tag )
{
	int i, found = FIELDS_NOTFOUND;
	intptr_t retn;

	for ( i=0; i<f->n && found==FIELDS_NOTFOUND; ++i ) {

		if ( !fields_match_level( f, i, level ) ) continue;
		if ( !fields_match_casetag( f, i, tag ) ) continue;

		if ( f->data[i].len!=0 ) found = i;
		else {
			if ( mode & FIELDS_NOLENOK_FLAG ) {
				return (void *) fields_null_value;
			} else if ( mode & FIELDS_SETUSE_FLAG ) {
				f->used[i] = 1; /* Suppress "noise" of unused */
			}
		}
	}

	if ( found==FIELDS_NOTFOUND ) return NULL;

	if ( mode & FIELDS_SETUSE_FLAG )
		fields_setused( f, found );

	if ( mode & FIELDS_STRP_FLAG )
		return (void *) &(f->data[found]);
	else if ( mode & FIELDS_POSP_FLAG ) {
		retn = found;
		return (void *) retn;
	} else
		return (void *) f->data[found].data;
}

void *
fields_findv_firstof( fields *f, int level, int mode, ... )
{
	char *tag, *value;
	va_list argp;

	va_start( argp, mode );
	while ( ( tag = ( char * ) va_arg( argp, char * ) ) ) {
		value = fields_findv( f, level, mode, tag );
		if ( value ) {
			va_end( argp );
			return value;
		}
	}
	va_end( argp );

	return NULL;
}

static int
fields_findv_each_add( fields *f, int mode, int n, vplist *a )
{
	int status;
	void *v;

	if ( n<0 || n>= f->n ) return FIELDS_OK;

	if ( mode & FIELDS_SETUSE_FLAG )
		fields_setused( f, n );

	if ( mode & FIELDS_STRP_FLAG ) {
		v = ( void * ) &( f->data[n] );
	} else if ( mode & FIELDS_POSP_FLAG ) {
		v = ( void * )( (long long) n );
	} else {
		v = ( void * ) str_cstr( &( f->data[n] ) );
	}

	status = vplist_add( a, v );

	if ( status==VPLIST_OK ) return FIELDS_OK;
	else return FIELDS_ERR;
}

int
fields_findv_each( fields *f, int level, int mode, vplist *a, const char *tag )
{
	int i, status;

	for ( i=0; i<f->n; ++i ) {

		if ( !fields_match_level( f, i, level ) ) continue;
		if ( !fields_match_casetag( f, i, tag ) ) continue;

		if ( f->data[i].len!=0 ) {
			status = fields_findv_each_add( f, mode, i, a );
			if ( status!=FIELDS_OK ) return status;
		} else {
			if ( mode & FIELDS_NOLENOK_FLAG ) {
				status = fields_findv_each_add( f, mode, i, a );
				if ( status!=FIELDS_OK ) return status;
			} else {
				f->used[i] = 1; /* Suppress "noise" of unused */
			}
		}

	}

	return FIELDS_OK;
}

static int
fields_build_tags( va_list argp, vplist *tags )
{
	int status;
	char *tag;

	while ( ( tag = ( char * ) va_arg( argp, char * ) ) ) {
		status = vplist_add( tags, tag );
		if ( status!=VPLIST_OK ) return FIELDS_ERR;
	}

	return FIELDS_OK;
}

static int
fields_match_casetags( fields *f, int n, vplist *tags )
{
	int i;

	for ( i=0; i<tags->n; ++i )
		if ( fields_match_casetag( f, n, vplist_get( tags, i ) ) ) return 1;

	return 0;
}

int
fields_findv_eachof( fields *f, int level, int mode, vplist *a, ... )
{
	int i, status;
	va_list argp;
	vplist tags;

	vplist_init( &tags );

	/* build list of tags to search for */
	va_start( argp, a );
	status = fields_build_tags( argp, &tags );
	va_end( argp );
	if ( status!=FIELDS_OK ) goto out;

	/* search list */
	for ( i=0; i<f->n; ++i ) {

		if ( !fields_match_level( f, i, level ) ) continue;
		if ( !fields_match_casetags( f, i, &tags ) ) continue;

		if ( f->data[i].len!=0 || ( mode & FIELDS_NOLENOK_FLAG ) ) {
			status = fields_findv_each_add( f, mode, i, a );
			if ( status!=FIELDS_OK ) goto out;
		} else {
			f->used[i] = 1; /* Suppress "noise" of unused */
		}

	}

out:
	vplist_free( &tags );
	return status;
}

void
fields_report( fields *f, FILE *fp )
{
	int i, n;
	n = fields_num( f );
	fprintf( fp, "# NUM   level = LEVEL   'TAG' = 'VALUE'\n" );
	for ( i=0; i<n; ++i ) {
		fprintf( stderr, "%d\tlevel = %d\t'%s' = '%s'\n",
			i+1,
			fields_level( f, i ),
			(char*)fields_tag( f, i, FIELDS_CHRP_NOUSE ),
			(char*)fields_value( f, i, FIELDS_CHRP_NOUSE )
		);
	}
}

