/*
 * str.c
 *
 * Version: 2018-09-21
 *
 * Copyright (c) Chris Putnam 1999-2019
 *
 * Source code released under the GPL version 2
 *
 *
 * routines for dynamically allocated strings
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <ctype.h>
#include <limits.h>
#include "is_ws.h"
#include "str.h"

/* Do not use asserts in STR_NOASSERT defined */
#ifdef STR_NOASSERT
#define NDEBUG
#endif
#include <assert.h>

#define str_initlen (64)

#ifndef STR_SMALL

#define str_clear_status( s ) s->status = STR_OK;
#define handle_memerr( s, f ) s->status = STR_MEMERR;
#define return_if_memerr( s ) \
{ \
	if ( s->status != STR_OK ) return; \
}
#define return_zero_if_memerr( s ) \
{ \
	if ( s->status != STR_OK ) return 0; \
}
#define return_after_delim_if_memerr( s, p, delim, finalstep ) \
{ \
	if ( s->status != STR_OK ) { \
		while ( p && *p && !strchr( delim, *p ) ) p++; \
		if ( p && *p && finalstep ) p++; \
		return p; \
	} \
}

#else

#define str_clear_status( s ) {}
#define handle_memerr( s, f ) \
{ \
	fprintf( stderr,"Error.  Cannot allocate memory in %s.\n", f ); \
	exit( EXIT_FAILURE ); \
}
#define return_if_memerr( s ) {}
#define return_zero_if_memerr( s ) {}
#define return_after_delim_if_memerr( s, p, delim, finalstep ) {}

#endif


/* Clear memory in resize/free if STR_PARANOIA defined */

#ifndef STR_PARANOIA

static void 
str_realloc( str *s, unsigned long minsize )
{
	unsigned long size;
	char *newptr;

	assert( s );
	return_if_memerr( s );

	size = 2 * s->dim;
	if (size < minsize) size = minsize;

	newptr = (char *) realloc( s->data, sizeof( *(s->data) )*size );
	if ( !newptr ) handle_memerr( s, __FUNCTION__ );

	s->data = newptr;
	s->dim = size;
}

/* define as a no-op */
#define str_nullify( s )

#else

static void 
str_realloc( str *s, unsigned long minsize )
{
	unsigned long size;
	char *newptr;

	assert( s );
	return_if_memerr( s );

	size = 2 * s->dim;
	if ( size < minsize ) size = minsize;

	newptr = (char *) malloc( sizeof( *(s->data) ) * size );
	if ( !newptr ) handle_memerr( s, __FUNCTION__ );

	if ( s->data ) {
		str_nullify( s );
		free( s->data );
	}
	s->data = newptr;
	s->dim = size;
}

static inline void
str_nullify( str *s )
{
	memset( s->data, 0, s->dim );
}

#endif

void 
str_init( str *s )
{
	assert( s );
	s->dim = 0;
	s->len = 0;
	s->data = NULL;
	str_clear_status( s );
}

void
str_initstr( str *s, str *from )
{
	assert( s );
	assert( from );
	str_init( s );
	str_strcpy( s, from );
}

void
str_initstrc( str *s, const char *initstr )
{
	assert( s );
	assert( initstr );
	str_init( s );
	str_strcpyc( s, initstr );
}

void
str_initstrsc( str *s, ... )
{
	const char *c;
	va_list ap;
	str_init( s );
	va_start( ap, s );
	do {
		c = va_arg( ap, const char * );
		if ( c ) str_strcatc( s, c );
	} while ( c );
	va_end( ap );
}

void
strs_init( str *s, ... )
{
	str *s2;
	va_list ap;
	str_init( s );
	va_start( ap, s );
	do {
		s2 = va_arg( ap, str * );
		if ( s2 ) str_init( s2 );
	} while ( s2 );
	va_end( ap );
}

int
str_memerr( str *s )
{
#ifndef STR_SMALL
	return s->status == STR_MEMERR;
#else
	return 0;
#endif
}

void
str_mergestrs( str *s, ... )
{
	va_list ap;
	const char *cp;
	str_clear_status( s );
	str_empty( s );
	va_start( ap, s );
	do {
		cp = va_arg( ap, const char * );
		if ( cp ) str_strcatc( s, cp );
	} while ( cp );
	va_end( ap );
}

static void 
str_initalloc( str *s, unsigned long minsize )
{
	unsigned long size = str_initlen;
	assert( s );
	if ( minsize > str_initlen ) size = minsize;
	s->data = (char *) malloc( sizeof( *(s->data) ) * size );
	if ( !s->data ) {
		fprintf(stderr,"Error.  Cannot allocate memory in str_initalloc, requested %lu characters.\n", size );
		exit( EXIT_FAILURE );
	}
	s->data[0]='\0';
	s->dim=size;
	s->len=0;
	str_clear_status( s );
}

str *
str_new( void )
{
	str *s = (str *) malloc( sizeof( *s ) );
	if ( s )
		str_initalloc( s, str_initlen );
	return s;
}

void 
str_free( str *s )
{
	assert( s );
	if ( s->data ) {
		str_nullify( s );
		free( s->data );
	}
	s->dim = 0;
	s->len = 0;
	s->data = NULL;
}

void
strs_free( str *s, ... )
{
	str *s2;
	va_list ap;
	str_free( s );
	va_start( ap, s );
	do {
		s2 = va_arg( ap, str * );
		if ( s2 ) str_free( s2 );
	} while ( s2 );
	va_end( ap );
}

void
str_delete( str *s )
{
	assert( s );
	str_free( s );
	free( s );
}

void
str_empty( str *s )
{
	assert( s );
	str_clear_status( s );
	if ( s->data ) {
		str_nullify( s );
		s->data[0] = '\0';
	}
	s->len = 0;
}

void
strs_empty( str *s, ... )
{
	str *s2;
	va_list ap;
	str_empty( s );
	va_start( ap, s );
	do {
		s2 = va_arg( ap, str * );
		if ( s2 ) str_empty( s2 );
	} while ( s2 );
	va_end( ap );
}

void
str_addchar( str *s, char newchar )
{
	assert( s );

	return_if_memerr( s );

	if ( newchar=='\0' ) return; /* appending '\0' is a null operation */

	if ( !s->data || s->dim==0 ) 
		str_initalloc( s, str_initlen );
	if ( s->len + 2 > s->dim ) 
		str_realloc( s, s->len*2 );

	s->data[s->len++] = newchar;
	s->data[s->len] = '\0';
}

/* str_addutf8
 *
 * Add potential multibyte character to s starting at pointer p.
 * Multibyte Unicode characters have the high bit set.
 *
 * Since we can progress more than one byte at p, return the
 * properly updated pointer p.
 */
const char *
str_addutf8( str *s, const char *p )
{
	if ( ! ((*p) & 128 ) ) {
		str_addchar( s, *p );
		p++;
	} else {
		while ( ((*p) & 128) ) {
			str_addchar( s, *p );
			p++;
		}
	}
	return p;
}

char *
str_cstr( str *s )
{
	assert( s );
	return s->data;
}

void 
str_fprintf( FILE *fp, str *s )
{
	assert( s );
	if ( s->data ) fprintf( fp, "%s", s->data );
}

void
str_prepend( str *s, const char *addstr )
{
	unsigned long lenaddstr, i;

	assert( s && addstr );

	return_if_memerr( s );

	lenaddstr = strlen( addstr );
	if ( lenaddstr==0 ) return; /* appending an empty string is a null op */

	if ( !s->data || !s->dim )
		str_initalloc( s, lenaddstr+1 );
	else {
		if ( s->len + lenaddstr  + 1 > s->dim )
			str_realloc( s, s->len + lenaddstr + 1 );
		for ( i=s->len+lenaddstr-1; i>=lenaddstr; i-- )
			s->data[i] = s->data[i-lenaddstr];
	}
	strncpy( s->data, addstr, lenaddstr );
	s->len += lenaddstr;
	s->data[ s->len ] = '\0';
}

static inline void
str_strcat_ensurespace( str *s, unsigned long n )
{
	unsigned long m = s->len + n + 1;
	if ( !s->data || !s->dim )
		str_initalloc( s, m );
	else if ( s->len + n + 1 > s->dim )
		str_realloc( s, m );
}

static inline void 
str_strcat_internal( str *s, const char *addstr, unsigned long n )
{
	return_if_memerr( s );
	str_strcat_ensurespace( s, n );
	strncat( &(s->data[s->len]), addstr, n );
	s->len += n;
	s->data[s->len]='\0';
}

void
str_strcat( str *s, str *from )
{
	assert ( s && from );
	if ( !from->data ) return;
	else str_strcat_internal( s, from->data, from->len );
}

void
str_strcatc( str *s, const char *from )
{
	unsigned long n;
	assert( s && from );
	n = strlen( from );
	str_strcat_internal( s, from, n );
}

void
str_segcat( str *s, char *startat, char *endat )
{
	unsigned long n;
	char *p;

	assert( s && startat && endat );
	assert( (size_t) startat < (size_t) endat );

	return_if_memerr( s );

	if ( startat==endat ) return;

	n = 0;
	p = startat;
	while ( p!=endat ) {
		n++;
		p++;
	}

	str_strcat_internal( s, startat, n );
}

void
str_indxcat( str *s, char *p, unsigned long start, unsigned long stop )
{
	unsigned long i;

	assert( s && p );
	assert( start <= stop );

	return_if_memerr( s );

	for ( i=start; i<stop; ++i )
		str_addchar( s, p[i] );
}

/* str_cpytodelim()
 *     term      = string of characters to be used as terminators
 *     finalstep = set to non-zero to position return value past the
 *                 terminating character
 */
const char *
str_cpytodelim( str *s, const char *p, const char *delim, unsigned char finalstep )
{
	assert( s );

	str_empty( s );
	return str_cattodelim( s, p, delim, finalstep );
}

/* str_cpytodelim()
 *     term      = string of characters to be used as terminators
 *     finalstep = set to non-zero to position return value past the
 *                 terminating character
 */
const char *
str_cattodelim( str *s, const char *p, const char *delim, unsigned char finalstep )
{
	assert( s );

	return_after_delim_if_memerr( s, p, delim, finalstep );

	while ( p && *p && !strchr( delim, *p ) ) {
		str_addchar( s, *p );
		p++;
	}
	if ( p && *p && finalstep ) p++;
	return p;
}

static inline void
str_strcpy_ensurespace( str *s, unsigned long n )
{
	unsigned long m = n + 1;
	if ( !s->data || !s->dim )
		str_initalloc( s, m );
	else if ( m > s->dim )
		str_realloc( s, m );
}

static inline void
str_strcpy_internal( str *s, const char *p, unsigned long n )
{
	return_if_memerr( s );

	str_strcpy_ensurespace( s, n );
	strncpy( s->data, p, n );
	s->data[n] = '\0';
	s->len = n;
}

void
str_strcpy( str *s, str *from )
{
	assert( s );
	assert( from );
	if ( s==from ) return;
	else if ( !from || from->len==0 ) str_empty( s );
	else str_strcpy_internal( s, from->data, from->len );
}

void 
str_strcpyc( str *s, const char *from )
{
	unsigned long n;
	assert( s && from );
	n = strlen( from );
	str_strcpy_internal( s, from, n );
}

/* str_segcpy( s, start, end );
 *
 * copies [start,end) into s
 */
void
str_segcpy( str *s, char *startat, char *endat )
{
	unsigned long n;
	char *p;

	assert( s && startat && endat );
	assert( ((size_t) startat) <= ((size_t) endat) );

	return_if_memerr( s );

	if ( startat==endat ) {
		str_empty( s );
		return;
	}

	n = 0;
	p = startat;
	while ( p!=endat ) {
		p++;
		n++;
	}

	str_strcpy_internal( s, startat, n );
}

/*
 * str_indxcpy( s, in, start, stop );
 *
 * copies in[start,stop) (excludes stop) into s
 */
void
str_indxcpy( str *s, char *p, unsigned long start, unsigned long stop )
{
	unsigned long i;

	assert( s && p );
	assert( start <= stop );

	return_if_memerr( s );

	if ( start == stop ) {
		str_empty( s );
		return;
	}
	str_strcpy_ensurespace( s, stop-start+1 );
	for ( i=start; i<stop; ++i )
		s->data[i-start] = p[i];
	s->len = stop-start;
	s->data[s->len] = '\0';
}

str *
str_strdup( str *from )
{
	str *s = str_new();
	if ( s )
		str_strcpy( s, from );
	return s;
}

str *
str_strdupc( const char *from )
{
	str *s = str_new();
	if ( s )
		str_strcpyc( s, from );
	return s;
}

void
str_segdel( str *s, char *p, char *q )
{
	str tmp1, tmp2;
	char *r;

	assert( s );

	return_if_memerr( s );

	r = &(s->data[s->len]);
	str_init( &tmp1 );
	str_init( &tmp2 );
	str_segcpy( &tmp1, s->data, p );
	str_segcpy( &tmp2, q, r );
	str_empty( s );
	if ( tmp1.data ) str_strcat( s, &tmp1 );
	if ( tmp2.data ) str_strcat( s, &tmp2 );
	str_free( &tmp2 );
	str_free( &tmp1 );
}

/*
 * str_findreplace()
 *
 *   if replace is "" or NULL, then delete find
 */

int
str_findreplace( str *s, const char *find, const char *replace )
{
	long diff;
	size_t findstart, searchstart;
	size_t p1, p2;
	size_t find_len, rep_len, curr_len;
	char empty[2] = "";
	unsigned long minsize;
	char *p;
	int n = 0;

	assert( s && find );

	return_zero_if_memerr( s );

	if ( !s->data || !s->dim ) return 0;
	if ( !replace ) replace = empty;

	find_len = strlen( find );
	rep_len  = strlen( replace );
	diff     = rep_len - find_len;
	if ( diff < 0 ) diff = 0;

	searchstart=0;
	while ((p=strstr(s->data + searchstart,find))!=NULL) {
		curr_len = strlen(s->data);
		findstart=(size_t) p - (size_t) s->data;
		minsize = curr_len + diff + 1;
		if (s->dim <= minsize) str_realloc( s, minsize );
		if ( find_len > rep_len ) {
			p1 = findstart + rep_len;
			p2 = findstart + find_len;
			while( s->data[p2] )
				s->data[p1++]=s->data[p2++];
			s->data[p1]='\0';
			n++;
		} else if ( find_len < rep_len ) {
			for ( p1=curr_len; p1>=findstart+find_len; p1-- )
				s->data[p1+diff] = s->data[p1];
			n++;
		}
		for (p1=0; p1<rep_len; p1++)
			s->data[findstart+p1]=replace[p1];
		searchstart = findstart + rep_len; 
		s->len += rep_len - find_len;
	}
	return n;
}


/* str_fget()
 *   returns 0 if we're done, 1 if we're not done
 *   extracts line by line (regardless of end characters)
 *   and feeds from buf....
 */
int
str_fget( FILE *fp, char *buf, int bufsize, int *pbufpos, str *outs )
{
	int  bufpos = *pbufpos, done = 0;
	char *ok;
	assert( fp && outs );
	str_empty( outs );
	while ( !done ) {
		while ( buf[bufpos] && buf[bufpos]!='\r' && buf[bufpos]!='\n' )
			str_addchar( outs, buf[bufpos++] );
		if ( buf[bufpos]=='\0' ) {
			ok = fgets( buf, bufsize, fp );
			bufpos=*pbufpos=0;
			if ( !ok && feof(fp) ) { /* end-of-file */
				buf[bufpos] = 0;
				if ( outs->len==0 ) return 0; /*nothing in out*/
				else return 1; /*one last out */
			}
		} else if ( buf[bufpos]=='\r' || buf[bufpos]=='\n' ) done=1;
	}
	if ( ( buf[bufpos]=='\n' && buf[bufpos+1]=='\r') ||
	     ( buf[bufpos]=='\r' && buf[bufpos+1]=='\n') ) bufpos+=2;
	else if ( buf[bufpos]=='\n' || buf[bufpos]=='\r' ) bufpos+=1; 
	*pbufpos = bufpos;
	return 1;
}

void
str_toupper( str *s )
{
	unsigned long i;
	assert( s );
	for ( i=0; i<s->len; ++i )
		s->data[i] = toupper( (unsigned char)s->data[i] );
}

void
str_tolower( str *s )
{
	unsigned long i;
	assert( s );
	for ( i=0; i<s->len; ++i )
		s->data[i] = tolower( (unsigned char)s->data[i] );
}

/* str_swapstrings( s1, s2 )
 * be sneaky and swap internal string data from one
 * string to another
 */
void
str_swapstrings( str *s1, str *s2 )
{
	char *tmpp;
	int tmp;

	assert( s1 && s2 );

	/* swap dimensioning info */
	tmp = s1->dim;
	s1->dim = s2->dim;
	s2->dim = tmp;

	/* swap length info */
	tmp = s1->len;
	s1->len = s2->len;
	s2->len = tmp;

	/* swap data */
	tmpp = s1->data;
	s1->data = s2->data;
	s2->data = tmpp;
}

void
str_trimstartingws( str *s )
{
	char *p, *q;
	int n;

	assert( s );

	if ( s->len==0 || !is_ws( s->data[0] ) ) return;

	n = 0;
	p = s->data;
	while ( is_ws( *p ) ) p++;

	q = s->data;
	while ( *p ) {
		*q++ = *p++;
		n++;
	}
	*q = '\0';

	s->len = n;
}

void
str_trimendingws( str *s )
{
	assert( s );
	while ( s->len > 0 && is_ws( s->data[s->len-1] ) ) {
		s->data[s->len-1] = '\0';
		s->len--;
	}
}

int
str_match_first( str *s, char ch )
{
	assert( s );
	if ( !s->len ) return 0;
	if ( s->data[0] == ch ) return 1;
	return 0;
}

int
str_match_end( str *s, char ch )
{
	assert( s );
	if ( !s->len ) return 0;
	if ( s->data[ s->len - 1 ] == ch ) return 1;
	return 0;
}

void
str_trimbegin( str *s, unsigned long n )
{
	char *p, *q;

	assert( s );

	if ( n==0 ) return;
	if ( s->len==0 ) return;
	if ( n >= s->len ) {
		str_empty( s );
		return;
	}

	p = s->data;
	while ( n-- > 0 ) p++;

	n = 0;
	q = s->data;
	while ( *p ) {
		*q++ = *p++;
		n++;
	}
	*q = '\0';

	s->len = n;
}

void
str_trimend( str *s, unsigned long n )
{
	assert( s );

	if ( n==0 ) return;
	if ( n >= s->len ) {
		str_empty( s );
		return;
	}

	s->len -= n;
	s->data[ s->len ] = '\0';
}

void
str_pad( str *s, unsigned long len, char ch )
{
	unsigned long i;
	assert( s );
	for ( i=s->len; i<len; i++ )
		str_addchar( s, ch );
}

void
str_copyposlen( str *s, str *in, unsigned long pos, unsigned long len )
{
	unsigned long i, max;
	assert( s );
	str_empty( s );
	max = pos+len;
	if ( max > in->len ) max = in->len;
	for ( i=pos; i<max; ++i )
		str_addchar( s, in->data[i] );
}

static void
str_check_case( str *s, int *lowercase, int *uppercase )
{
	int i;
	assert( s );
	*lowercase = 0;
	*uppercase = 0;
	if ( s->len < 1 ) return;
	for ( i=0; i<s->len && !( *lowercase && *uppercase ); ++i ) {
		if ( isalpha( (unsigned char)s->data[i] ) ) {
			if ( isupper( (unsigned char)s->data[i] ) ) *uppercase += 1;
			else if ( islower( (unsigned char)s->data[i] ) ) *lowercase += 1;
		}
	}
}

int
str_is_mixedcase( str *s )
{
	int lowercase, uppercase;
	str_check_case( s, &lowercase, &uppercase );
	if ( lowercase > 0 && uppercase > 0 ) return 1;
	return 0;
}

int
str_is_lowercase( str *s )
{
	int lowercase, uppercase;
	str_check_case( s, &lowercase, &uppercase );
	if ( lowercase > 0 && uppercase == 0 ) return 1;
	return 0;
}

int
str_is_uppercase( str *s )
{
	int lowercase, uppercase;
	str_check_case( s, &lowercase, &uppercase );
	if ( lowercase == 0 && uppercase > 0 ) return 1;
	return 0;
}

void
str_stripws( str *s )
{
	unsigned long len = 0;
	char *p, *q;
	assert( s );
	if ( s->len ) {
		p = q = s->data;
		while ( *p ) {
			if ( !is_ws( *p ) ) {
				*q = *p;
				q++;
				len++;
			}
			p++;
		}
		*q = '\0';
	}
	s->len = len;
}

int
str_strcmp( const str *s, const str *t )
{
	assert( s );
	assert( t );
	if ( s->len == 0 && t->len == 0 ) return 0;
	if ( s->len == 0 ) return strcmp( "", t->data );
	if ( t->len == 0 ) return strcmp( s->data, "" );
	return strcmp( s->data, t->data );
}

int
str_strcmpc( const str *s, const char *t )
{
	assert( s );
	assert( t );
	if ( s->len == 0 ) return strcmp( "", t );
	return strcmp( s->data, t );
}

int
str_strncmp( const str *s, const str *t, size_t n )
{
	assert( s );
	assert( t );
	if ( s->len == 0 && t->len == 0 ) return 0;
	if ( s->len == 0 ) return strncmp( "", t->data, n );
	if ( t->len == 0 ) return strncmp( s->data, "", n );
	return strncmp( s->data, t->data, n );
}

int
str_strncmpc( const str *s, const char *t, size_t n )
{
	assert( s );
	assert( t );
	if ( s->len == 0 ) return strncmp( "", t, n );
	return strncmp( s->data, t, n );
}

int
str_strcasecmp( const str *s, const str *t )
{
	assert( s );
	assert( t );
	if ( s->len == 0 && t->len == 0 ) return 0;
	if ( s->len == 0 ) return strcasecmp( "", t->data );
	if ( t->len == 0 ) return strcasecmp( s->data, "" );
	return strcasecmp( s->data, t->data );
}

int
str_strcasecmpc( const str *s, const char *t )
{
	assert( s );
	assert( t );
	if ( s->len == 0 ) return strcasecmp( "", t );
	return strcasecmp( s->data, t );
}

char *
str_strstr( const str *s, const str *t )
{
	assert( s );
	assert( t );
	if ( s->len == 0 && t->len == 0 ) return strstr( "", "" );
	if ( s->len == 0 ) return strstr( "", t->data );
	if ( t->len == 0 ) return strstr( s->data, "" );
	return strstr( s->data, t->data );
}

char *
str_strstrc( const str *s, const char *t )
{
	assert( s );
	assert( t );
	if ( s->len == 0 ) return strstr( "", t );
	return strstr( s->data, t );
}

void
str_reverse( str *s )
{
	unsigned long i, max;
	char tmp;
	assert( s );
	max = s->len / 2;
	for ( i=0; i<max; ++i ) {
		tmp = s->data[ i ];
		s->data[ i ] = s->data[ s->len - 1 - i ];
		s->data[ s->len - 1 - i ] = tmp;
	}
}

int
str_fgetline( str *s, FILE *fp )
{
	int ch, eol = 0;
	assert( s );
	assert( fp );
	str_empty( s );
	if ( feof( fp ) ) return 0;
	while ( !feof( fp ) && !eol ) {
		ch = fgetc( fp );
		if ( ch == EOF ) {
			if ( s->len ) return 1;
			else return 0;
		}
		else if ( ch == '\n' ) eol = 1;
		else if ( ch == '\r' ) {
			ch = fgetc( fp );
			if ( ch != '\n' ) ungetc( ch, fp );
			eol = 1;
		} else {
			str_addchar( s, (char) ch );
		}
	}
	return 1;
}

/*
 * s = "Hi!\0", s.len = 3
 *
 * str_char( s, 0 ) = 'H'  str_revchar( s, 0 ) = '!'
 * str_char( s, 1 ) = 'i'  str_revchar( s, 1 ) = 'i'
 * str_char( s, 2 ) = '!'  str_revchar( s, 2 ) = 'H'
 * str_char( s, 3 ) = '\0' str_revchar( s, 3 ) = '\0'
 */
char
str_char( str *s, unsigned long n )
{
	assert( s );
	if ( s->len==0 || n >= s->len ) return '\0';
	return s->data[ n ];
}

char
str_revchar( str *s, unsigned long n )
{
	assert( s );
	if ( s->len==0 || n >= s->len ) return '\0';
	return s->data[ s->len - n - 1];
}

void
str_makepath( str *path, const char *dirname, const char *filename, char sep )
{
	assert( path );
	if ( dirname ) str_strcpyc( path, dirname );
	else str_empty( path );

	if ( path->len && path->data[path->len-1]!=sep )
		str_addchar( path, sep );

	if ( filename ) str_strcatc( path, filename );
}

void
str_fill( str *s, unsigned long n, char fillchar )
{
	unsigned long i;
	assert( s );
	str_clear_status( s );
	if ( !s->data || s->dim==0 )
		str_initalloc( s, n+1 );
	if ( n + 1 > s->dim )
		str_realloc( s, n+1 );
	for ( i=0; i<n; ++i )
		s->data[i] = fillchar;
	s->data[n] = '\0';
	s->len = n;
}

int
str_has_value( str *s )
{
	if ( !s || s->len==0 ) return 0;
	return 1;
}

int
str_is_empty( str *s )
{
	if ( !s || s->len==0 ) return 1;
	return 0;
}

unsigned long
str_strlen( str *s )
{
	if ( !s ) return 0;
	else return s->len;
}
