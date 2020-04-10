/*
 * slist.c
 *
 * version: 2019-01-14
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 * Implements a simple managed array of strs.
 *
 */
#include "slist.h"

/* Do not use asserts in VPLIST_NOASSERT defined */
#ifdef VPLIST_NOASSERT
#define NDEBUG
#endif
#include <assert.h>

#define SLIST_MINALLOC (20)

#define SLIST_EXACT_SIZE  (0)
#define SLIST_DOUBLE_SIZE (1)

/*
 * returns 1 if n is valid string in slist
 */
static inline int
slist_valid_num( slist *a, slist_index n )
{
	if ( n < 0 || n >= a->n ) return 0;
	return 1;
}

void
slist_init( slist *a  )
{
	assert( a );

	a->strs = NULL;
	a->max = 0;
	a->n = 0;
	a->sorted = 1;
}

int
slist_init_values( slist *a, ... )
{
	int status = SLIST_OK;
	va_list ap;
	str *s;

	slist_init( a );

	va_start( ap, a );
	do {
		s = va_arg( ap, str * );
		if ( s ) {
			status = slist_add( a, s );
			if ( status!=SLIST_OK ) goto out;
		}
	} while ( s );
out:
	va_end( ap );

	return status;
}

int
slist_init_valuesc( slist *a, ... )
{
	int status = SLIST_OK;
	va_list ap;
	char *s;

	slist_init( a );

	va_start( ap, a );
	do {
		s = va_arg( ap, char * );
		if ( s ) {
			status = slist_addc( a, s );
			if ( status!=SLIST_OK ) goto out;
		}
	} while ( s );
out:
	va_end( ap );

	return status;
}

void
slist_empty( slist *a )
{
	slist_index i;

	assert( a );

	for ( i=0; i<a->max; ++i )
		str_empty( &(a->strs[i]) );

	a->n = 0;
	a->sorted = 1;
}

void
slist_free( slist *a )
{
	slist_index i;

	assert( a );

	for ( i=0; i<a->max; ++i )
		str_free( &(a->strs[i]) );

	free( a->strs );
	slist_init( a );
}

slist *
slist_new( void )
{
	slist *a;

	a = ( slist * ) malloc( sizeof ( slist ) );
	if ( a ) slist_init( a );

	return a;
}

void
slist_delete( slist *a )
{
	assert( a );

	slist_free( a );
	free( a );
}

void
slist_deletev( void *v )
{
	slist_delete( (slist*) v );
}

void
slist_swap( slist *a, slist_index n1, slist_index n2 )
{
	assert( a );

	if ( slist_valid_num( a, n1 ) && slist_valid_num( a, n2 ) )
		str_swapstrings( &(a->strs[n1]), &(a->strs[n2]) );
}

static int
slist_revcomp( const void *v1, const void *v2 )
{
	str *s1 = ( str *) v1;
	str *s2 = ( str *) v2;
	int n;

	if ( !s1->len && !s2->len ) return 0;
	else if ( !s1->len ) return 1;
	else if ( !s2->len ) return -1;

	n = str_strcmp( s1, s2 );
	if ( n==0 ) return 0;
	else if ( n > 0 ) return -1;
	else return 1;
}

static int
slist_comp( const void *v1, const void *v2 )
{
	str *s1 = ( str *) v1;
	str *s2 = ( str *) v2;
	if ( !s1->len && !s2->len ) return 0;
	else if ( !s1->len ) return -1;
	else if ( !s2->len ) return 1;
	else return str_strcmp( s1, s2 );
}

static int
slist_comp_step( slist *a, slist_index n1, slist_index n2 )
{
	return slist_comp( (const void*) &(a->strs[n1]), (const void*) &(a->strs[n2]) );
}

static str *
slist_set_cleanup( slist *a, slist_index n )
{
	if ( str_memerr( &(a->strs[n]) ) ) return NULL;
	if ( a->sorted ) {
		if ( n>0 && slist_comp_step( a, n-1, n )>0 )
			a->sorted = 0;
	}
	if ( a->sorted ) {
		if ( n<a->n-1 && slist_comp_step( a, n, n+1 )>0 )
			a->sorted = 0;
	}
	return &(a->strs[n]);
}

str *
slist_setc( slist *a, slist_index n, const char *s )
{
	assert( a );
	assert( s );

	if ( !slist_valid_num( a, n ) ) return NULL;
	str_strcpyc( &(a->strs[n]), s );
	return slist_set_cleanup( a, n );
}

str *
slist_set( slist *a, slist_index n, str *s )
{
	assert( s );

	return slist_setc( a, n, str_cstr( s ) );
}

/*
 * return pointer to str 'n'
 */
str *
slist_str( slist *a, slist_index n )
{
	assert( a );

	if ( !slist_valid_num( a, n ) ) return NULL;
	else return &(a->strs[n]);
}

/*
 * return pointer to C string 'n'
 *
 * So long as the index is a valid number ensure
 * that a pointer is returned even if the newstr isn't
 * allocated. Only return NULL if the index
 * is invalid. Thus we can convert loops like:
 *
 * for ( i=0; i<a->n; ++i ) {
 *      p = slist_cstr( a, i );
 *      if ( p==NULL ) continue; // empty string
 *      ...
 * }
 *
 * to
 *
 * i = 0;
 * while ( ( p = slist_cstr( a, i ) ) ) {
 *      ...
 *      i++;
 * }
 *
 */
char *
slist_cstr( slist *a, slist_index n )
{
	static char empty[] = "";
	char *p;

	assert( a );

	if ( !slist_valid_num( a, n ) ) return NULL;
	p = str_cstr( &(a->strs[n]) );
	if ( p ) return p;
	else return empty;
}

static inline int
slist_alloc( slist *a, slist_index alloc )
{
	slist_index i;

	a->strs = ( str* ) malloc( sizeof( str ) * alloc );
	if ( !(a->strs) ) return SLIST_ERR_MEMERR;

	a->max = alloc;
	a->n   = 0;

	for ( i=0; i<alloc; ++i )
		str_init( &(a->strs[i]) );

	return SLIST_OK;
}

static inline int
slist_realloc( slist *a, slist_index alloc )
{
	slist_index i;
	str *more;

	more = ( str* ) realloc( a->strs, sizeof( str ) * alloc );
	if ( !more ) return SLIST_ERR_MEMERR;

	a->strs = more;

	for ( i=a->max; i<alloc; ++i )
		str_init( &(a->strs[i]) );

	a->max = alloc;

	return SLIST_OK;
}

#define SLIST_EXACT_SIZE  (0)
#define SLIST_DOUBLE_SIZE (1)

static int
slist_ensure_space( slist *a, slist_index n, int mode )
{
	int status = SLIST_OK;
	int alloc = n;

	if ( a->max==0 ) {
		if ( mode == SLIST_DOUBLE_SIZE && alloc < SLIST_MINALLOC ) alloc = SLIST_MINALLOC;
		status = slist_alloc( a, alloc );
	}

	else if ( a->max < n ) {
		if ( mode == SLIST_DOUBLE_SIZE && alloc < a->max * 2 ) alloc = a->max * 2;
		status = slist_realloc( a, alloc );
	}

	return status;
}

int
slist_addvp( slist *a, int mode, void *vp )
{
	str *s = NULL;
	int status;

	status = slist_ensure_space( a, a->n+1, SLIST_DOUBLE_SIZE );

	if ( status==SLIST_OK ) {

		s = &( a->strs[a->n] );

		if ( mode==SLIST_CHR )
			str_strcpyc( s, (const char*) vp );
		else
			str_strcpy( s, (str*) vp );

		if ( str_memerr( s ) ) return SLIST_ERR_MEMERR;
		a->n++;
		if ( a->sorted && a->n > 1 ) {
			if ( slist_comp_step( a, a->n-2, a->n-1 ) > 0 )
				a->sorted = 0;
		}

	}

	return SLIST_OK;
}
int
slist_addc( slist *a, const char *s )
{
	return slist_addvp( a, SLIST_CHR, (void*)s );
}
int
slist_add( slist *a, str *s )
{
	return slist_addvp( a, SLIST_STR, (void*)s );
}

int
slist_addvp_ret( slist *a, int mode, void *vp, int retok, int reterr )
{
	int status = slist_addvp( a, mode, vp );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}
int
slist_addc_ret( slist *a, const char *value, int retok, int reterr )
{
	int status = slist_addc( a, value );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}
int
slist_add_ret( slist *a, str *value, int retok, int reterr )
{
	int status = slist_add( a, value );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}

int
slist_addvp_unique( slist *a, int mode, void *vp )
{
	int n;

	if ( mode==SLIST_CHR )
		n = slist_findc( a, (const char*) vp );
	else
		n = slist_find( a, (str*) vp );

	if ( slist_wasfound( a, n ) )
		return SLIST_OK;
	else
		return slist_addvp( a, mode, vp );
}
int
slist_addc_unique( slist *a, const char *s )
{
	return slist_addvp_unique( a, SLIST_CHR, (void*)s );
}
int
slist_add_unique( slist *a, str *s )
{
	return slist_addvp_unique( a, SLIST_STR, (void*)s );
}

int
slist_addvp_unique_ret( slist *a, int mode, void *vp, int retok, int reterr )
{
	int status = slist_addvp_unique( a, mode, vp );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}
int
slist_addc_unique_ret( slist *a, const char *s, int retok, int reterr )
{
	int status = slist_addc_unique( a, s );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}
int
slist_add_unique_ret( slist *a, str *s, int retok, int reterr )
{
	int status = slist_add_unique( a, s );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}

int
slist_addvp_all( slist *a, int mode, ... )
{
	int status = SLIST_OK;
	va_list ap;
	void *v;

	va_start( ap, mode );

	do {

		if ( mode==SLIST_CHR )
			v = va_arg( ap, char * );
		else
			v = va_arg( ap, str * );

		if ( v ) {
			status = slist_addvp( a, mode, v );
			if ( status!=SLIST_OK ) goto out;
		}

	} while ( v );

out:
	va_end( ap );
	return status;
}

int
slist_add_all( slist *a, ... )
{
	int status = SLIST_OK;
	va_list ap;
	str *v;

	va_start( ap, a );

	do {
		v = va_arg( ap, str * );

		if ( v ) {
			status = slist_addvp( a, SLIST_STR, (void*)v );
			if ( status!=SLIST_OK ) goto out;
		}

	} while ( v );
out:
	va_end( ap );
	return status;
}

int
slist_addc_all( slist *a, ... )
{
	int status = SLIST_OK;
	const char *v;
	va_list ap;

	va_start( ap, a );

	do {

		v = va_arg( ap, const char * );

		if ( v ) {
			status = slist_addvp( a, SLIST_CHR, (void*)v );
			if ( status!=SLIST_OK ) goto out;
		}

	} while ( v );
out:
	va_end( ap );
	return status;
}

int
slist_append( slist *a, slist *toadd )
{
	int i, status;

	assert( a );
	assert( toadd );

	status = slist_ensure_space( a, a->n + toadd->n, SLIST_EXACT_SIZE );

	if ( status == SLIST_OK ) {

		for ( i=0; i<toadd->n; ++i ) {
			str_strcpy( &(a->strs[a->n+i]), &(toadd->strs[i]) );
			if ( str_memerr( &(a->strs[a->n+i]) ) ) return SLIST_ERR_MEMERR;
		}

		if ( a->sorted && toadd->sorted == 0 ) a->sorted = 0;
		if ( a->sorted && a->n > 0 ) {
			if ( slist_comp_step( a, a->n-1, a->n ) > 0 ) {
				a->sorted = 0;
			}
		}

		a->n += toadd->n;

	}

	return status;
}

int
slist_append_unique( slist *a, slist *toadd )
{
	int i, status;

	assert( a );
	assert( toadd );

	for ( i=0; i<toadd->n; ++i ) {
		status = slist_add_unique( a, &(toadd->strs[i]) );
		if ( status!=SLIST_OK ) return status;
	}

	return SLIST_OK;
}

int
slist_append_ret( slist *a, slist *toadd, int retok, int reterr )
{
	int status;

	status = slist_append( a, toadd );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}

int
slist_append_unique_ret( slist *a, slist *toadd, int retok, int reterr )
{
	int status;

	status = slist_append_unique( a, toadd );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}

int
slist_remove( slist *a, slist_index n )
{
	int i;

	assert( a );

	if ( !slist_valid_num( a, n ) ) return SLIST_ERR_BADPARAM;

	for ( i=n+1; i<a->n; ++i ) {
		str_strcpy( &(a->strs[i-1]), &(a->strs[i]) );
		if ( str_memerr( &(a->strs[i-1]) ) ) return SLIST_ERR_MEMERR;
	}

	a->n--;

	return SLIST_OK;
}

void
slist_sort( slist *a )
{
	qsort( a->strs, a->n, sizeof( str ), slist_comp );
	a->sorted = 1;
}

void
slist_revsort( slist *a )
{
	qsort( a->strs, a->n, sizeof( str ), slist_revcomp );
	a->sorted = 0;
}

static slist_index
slist_find_sorted( slist *a, const char *searchstr )
{
	slist_index min, max, mid;
	str s, *cs;
	int comp;

	assert( a );
	assert( searchstr );

	str_initstrc( &s, searchstr );
	min = 0;
	max = a->n - 1;
	while ( min <= max ) {
		mid = ( min + max ) / 2;
		cs = slist_str( a, mid );
		comp = slist_comp( (void*)cs, (void*) (&s) );
		if ( comp==0 ) {
			str_free( &s );
			return mid;
		}
		else if ( comp > 0 ) max = mid - 1;
		else if ( comp < 0 ) min = mid + 1;
	}
	str_free( &s );
	return -1;
}

static slist_index
slist_find_simple( slist *a, const char *searchstr, int nocase )
{
	slist_index i;

	assert( a );
	assert( searchstr );

	if ( nocase ) {
		for ( i=0; i<a->n; ++i )
			if ( !str_strcasecmpc( &(a->strs[i]), searchstr ) )
				return i;
	} else {
		for ( i=0; i<a->n; ++i )
			if ( !str_strcmpc( &(a->strs[i]), searchstr ) )
				return i;
	}
	return -1;
}

slist_index
slist_findc( slist *a, const char *searchstr )
{
	assert( a );

	if ( a->n==0 ) return -1;
	if ( a->sorted )
		return slist_find_sorted( a, searchstr );
	else
		return slist_find_simple( a, searchstr, 0 );
}

slist_index
slist_find( slist *a, str *searchstr )
{
	if ( searchstr->len==0 ) return -1;
	return slist_findc( a, str_cstr( searchstr ) );
}

slist_index
slist_findnocasec( slist *a, const char *searchstr )
{
	assert( a );

	return slist_find_simple( a, searchstr, 1 );
}

slist_index
slist_findnocase( slist *a, str *searchstr )
{
	if ( searchstr->len==0 ) return -1;
	return slist_findnocasec( a, str_cstr( searchstr ) );
}

int
slist_wasfound( slist *a, slist_index n )
{
	return ( n!=-1 );
}

int
slist_wasnotfound( slist *a, slist_index n )
{
	return ( n==-1 );
}

int
slist_fillfp( slist *a, FILE *fp, unsigned char skip_blank_lines )
{
	int status, ret = SLIST_OK;
	str line;

	assert( a );
	assert( fp );

	slist_empty( a );
	str_init( &line );

	while ( str_fgetline( &line, fp ) ) {
		if ( skip_blank_lines && line.len==0 ) continue;
		status = slist_add( a, &line );
		if ( status!=SLIST_OK ) {
			ret = SLIST_ERR_MEMERR;
			goto out;
		}
	}

out:
	str_free( &line );
	return ret;
}

int
slist_fill( slist *a, const char *filename, unsigned char skip_blank_lines )
{
	FILE *fp;
	int ret;

	fp = fopen( filename, "r" );
	if ( !fp ) return SLIST_ERR_CANTOPEN;

	ret = slist_fillfp( a, fp, skip_blank_lines );

	fclose( fp );

	return ret;
}

int
slist_copy( slist *to, slist *from )
{
	slist_index i;
	int status;

	assert( to );
	assert( from );

	slist_free( to );

	if ( from->n==0 ) return SLIST_OK;

	status = slist_ensure_space( to, from->n, SLIST_EXACT_SIZE );

	if ( status == SLIST_OK ) {

		to->sorted = from->sorted;
		to->n      = from->n;

		for ( i=0; i<from->n; i++ ) {
			str_strcpy( &(to->strs[i]), &(from->strs[i]) );
			if ( str_memerr( &(to->strs[i]) ) ) return SLIST_ERR_MEMERR;
		}

	}
	return SLIST_OK;
}

int
slist_copy_ret( slist *to, slist *from, int retok, int reterr )
{
	int status = slist_copy( to, from );
	if ( status==SLIST_OK ) return retok;
	else return reterr;
}

slist *
slist_dup( slist *from )
{
	int status;
	slist *to;

	to = slist_new();
	if ( to ) {
		status = slist_copy( to, from );
		if ( status!=SLIST_OK ) {
			slist_delete( to );
			to = NULL;
		}
	}

	return to;
}

unsigned long
slist_get_maxlen( slist *a )
{
	unsigned long max = 0;
	slist_index i;
	str *s;

	assert( a );

	for ( i=0; i<a->n; ++i ) {
		s = slist_str( a, i );
		if ( s->len > max ) max = s->len;
	}

	return max;
}

void
slist_dump( slist *a, FILE *fp, int newline )
{
	slist_index i;

	assert( a );
	assert( fp );

	if ( newline ) {
		for ( i=0; i<a->n; ++i )
			fprintf( fp, "%s\n", slist_cstr( a, i ) );
	}

	else {
		for ( i=0; i<a->n; ++i )
			fprintf( fp, "%s", slist_cstr( a, i ) );
	}
}

int
slist_match_entry( slist *a, int n, const char *s )
{
	assert( a );

	if ( !slist_valid_num( a, n ) ) return 0;
	if ( str_strcmpc( &(a->strs[n]), s ) ) return 0;
	return 1;
}

void
slist_trimend( slist *a, int n )
{
	slist_index i;

	assert( a );

	if ( a->n - n < 1 ) {
		slist_empty( a );
	} else {
		for ( i=a->n -n; i<a->n; ++i ) {
			str_empty( &(a->strs[i]) );
		}
		a->n -= n;
	}
}

int
slist_tokenizec( slist *tokens, char *p, const char *delim, int merge_delim )
{
	int status, ret = SLIST_OK;
	char *q;
	str s;

	assert( tokens );

	slist_empty( tokens );
	str_init( &s );
	while ( p && *p ) {
		q = p;
		while ( *q && !strchr( delim, *q ) ) q++;
		str_segcpy( &s, p, q );
		if ( str_memerr( &s ) ) { ret = SLIST_ERR_MEMERR; goto out; }
		if ( s.len ) {
			status = slist_addvp( tokens, SLIST_STR, (void*) &s );
			if ( status!=SLIST_OK ) { ret = SLIST_ERR_MEMERR; goto out; }
		} else if ( !merge_delim ) {
			status = slist_addvp( tokens, SLIST_CHR, (void*) "" );
			if ( status!=SLIST_OK ) { ret = SLIST_ERR_MEMERR; goto out; }
		}
		p = q;
		if ( *p ) p++;
	}
out:
	str_free( &s );
	return ret;
}

int
slist_tokenize( slist *tokens, str *in, const char *delim, int merge_delim )
{
	return slist_tokenizec( tokens, str_cstr( in ), delim, merge_delim );
}

void
slists_init( slist *a, ... )
{
	slist *a2;
	va_list ap;
	slist_init( a );
	va_start( ap, a );
	do {
		a2 = va_arg( ap, slist * );
		if ( a2 ) slist_init( a2 );
	} while ( a2 );
	va_end( ap );
}

void
slists_free( slist *a, ... )
{
	slist *a2;
	va_list ap;
	slist_free( a );
	va_start( ap, a );
	do {
		a2 = va_arg( ap, slist * );
		if ( a2 ) slist_free( a2 );
	} while ( a2 );
	va_end( ap );
}

void
slists_empty( slist *a, ... )
{
	slist *a2;
	va_list ap;
	slist_empty( a );
	va_start( ap, a );
	do {
		a2 = va_arg( ap, slist * );
		if ( a2 ) slist_empty( a2 );
	} while ( a2 );
	va_end( ap );
}
