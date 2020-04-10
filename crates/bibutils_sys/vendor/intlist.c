/*
 * intlist.c
 *
 * Copyright (c) Chris Putnam 2007-2019
 *
 * Version 1/12/2017
 *
 * Source code released under the GPL version 2
 *
 * Implements a simple managed array of ints
 *
 */
#include <stdlib.h>
#include <assert.h>
#include "intlist.h"

#define INTLIST_MINALLOC (20)

static int
intlist_validn( intlist *il, int n )
{
	if ( n < 0 || n >= il->n ) return 0;
	return 1;
}

int
intlist_wasfound( intlist *il, int n )
{
	if ( n!=-1 ) return 1;
	else return 0;
}

int
intlist_wasnotfound( intlist *il, int n )
{
	if ( n==-1 ) return 1;
	else return 0;
}

static int
intlist_alloc( intlist *il, int alloc_size )
{
	il->data = ( int * ) calloc( alloc_size, sizeof( int ) );
	if ( !(il->data) ) return INTLIST_MEMERR;
	il->max = alloc_size;
	il->n   = 0;
	return INTLIST_OK;
}

static int
intlist_realloc( intlist *il, int alloc_size )
{
	int i, *more;

	more = ( int * ) realloc( il->data, sizeof( int ) * alloc_size );
	if ( !more ) return INTLIST_MEMERR;

	il->data = more;
	il->max  = alloc_size;

	for ( i=il->max; i<alloc_size; ++i )
		il->data[i] = 0;

	return INTLIST_OK;
}

static int
intlist_ensure_space( intlist *il, int n )
{
	int alloc = n;

	if ( il->max == 0 ) {
		if ( alloc < INTLIST_MINALLOC ) alloc = INTLIST_MINALLOC;
		return intlist_alloc( il, alloc );
	}

	else if ( il->max <= n ) {
		if ( alloc < il->max * 2 ) alloc = il->max * 2;
		return intlist_realloc( il, alloc );
	}

	return INTLIST_OK;
}

/* intlist_add()
 *
 * Returns INTLIST_OK/INTLIST_MEMERR
 */
int
intlist_add( intlist *il, int value )
{
	int status;

	assert( il );

	status = intlist_ensure_space( il, il->n+1 );

	if ( status == INTLIST_OK ) {
		il->data[ il->n ] = value;
		il->n++;
	}

	return status;
}

/* intlist_add_unique()
 *
 * Returns INTLIST_OK/INTLIST_MEMERR
 */
int
intlist_add_unique( intlist *il, int value )
{
	int n;

	assert( il );

	n = intlist_find( il, value );
	if ( intlist_wasnotfound( il, n ) )
		return intlist_add( il, value );
	else
		return INTLIST_OK;
}

int
intlist_find_or_add( intlist *il, int value )
{
	int n, status;

	n = intlist_find( il, value );

	if ( intlist_wasfound( il, n ) ) {
		return n;
	}

	else {
		status = intlist_add( il, value );
		if ( status!=INTLIST_OK ) return -1;
		else return il->n - 1;
	}
}

/* intlist_find()
 *
 * Returns position of value in range [0,n), or -1 if
 * value cannot be found
 */
int
intlist_find( intlist *il, int value )
{
	int i;

	assert( il );

	for ( i=0; i<il->n; ++i )
		if ( il->data[i]==value ) return i;

	return -1;
}

static int
intlist_remove_pos_core( intlist *il, int pos )
{
	int i;

	assert( il );

	for ( i=pos; i<il->n-1; ++i )
		il->data[i] = il->data[i+1];
	il->n -= 1;

	return INTLIST_OK;
}

/* intlist_remove_pos()
 *
 * Returns INTLIST_OK on success.
 */
int
intlist_remove_pos( intlist *il, int pos )
{
	assert( il );
	assert( intlist_validn( il, pos ) );

	return intlist_remove_pos_core( il, pos );
}

/* intlist_remove()
 *
 * Removes first instance of value from the intlist.
 * Returns INTLIST_OK/INTLIST_VALUE_MISSING
 */
int
intlist_remove( intlist *il, int value )
{
	int pos;

	assert( il );

	pos = intlist_find( il, value );
	if ( pos==-1 ) return INTLIST_VALUE_MISSING;

	return intlist_remove_pos_core( il, pos );
}

/* don't actually free space, just reset counter */
void
intlist_empty( intlist *il )
{
	assert( il );

	il->n = 0;
}

void
intlist_free( intlist *il )
{
	assert( il );

	if ( il->data ) free( il->data );
	intlist_init( il );
}

void
intlist_delete( intlist *il )
{
	assert( il );

	if ( il->data ) free( il->data );
	free( il );
}

void
intlist_init( intlist *il  )
{
	assert( il );

	il->data = NULL;
	il->max = 0;
	il->n = 0;
}

/* Returns INTLIST_OK/INTLIST_MEMERR
 */
int
intlist_init_fill( intlist *il, int n, int v )
{
	intlist_init( il );
	return intlist_fill( il, n, v );
}

/* intlist_init_range()
 *
 * Initializes intlist to values from [low,high) with step step.
 * Returns INTLIST_OK/INTLIST_MEMERR.
 */
int
intlist_init_range( intlist *il, int low, int high, int step )
{
	intlist_init( il );
	return intlist_fill_range( il, low, high, step );
}

/* intlist_new()
 *
 * Allocates an empty intlist.
 * Returns pointer to intlist on success, NULL on memory error.
 */
intlist *
intlist_new( void )
{
	intlist *il;
	il = ( intlist * ) malloc( sizeof( intlist ) );
	if ( il ) intlist_init( il );
	return il;
}

/* intlist_new_range()
 *
 * Allocates a intlist initialized to values from [low,high) in increments of step.
 * Returns pointer to intlist on success, NULL on memory error.
 */
intlist *
intlist_new_range( int low, int high, int step )
{
	intlist *il;
	int status;

	il = intlist_new();
	if ( il ) {
		status = intlist_fill_range( il, low, high, step );
		if ( status==INTLIST_MEMERR ) {
			intlist_free( il );
			free( il );
			il = NULL;
		}
	}
	return il;
}

/* intlist_new_range()
 *
 * Allocates a intlist initialized to n elements with value v.
 * Returns pointer to intlist on success, NULL on memory error.
 */
intlist *
intlist_new_fill( int n, int v )
{
	intlist *il;
	int status;

	il = intlist_new();
	if ( il ) {
		status = intlist_fill( il, n, v );
		if ( status==INTLIST_MEMERR ) {
			intlist_free( il );
			free( il );
			il = NULL;
		}
	}
	return il;
}

/* intlist_fill()
 *
 * Fill an intlist with n elements of value v.
 *
 * Returns INTLIST_OK or INTLIST_MEMERR.
 */
int
intlist_fill( intlist *il, int n, int v )
{
	int i, status;

	assert ( n > 0 );

	status = intlist_ensure_space( il, n );

	if ( status==INTLIST_OK ) {

		for ( i=0; i<n; ++i )
			il->data[i] = v;

		il->n = n;

	}

	return status;
}

/* intlist_fill_range()
 *
 * Fill an intlist with the values [low,high) in increments of step
 *
 * Returns INTLIST_OK or INTLIST_MEMERR.
 */
int
intlist_fill_range( intlist *il, int low, int high, int step )
{
	int i, n, status;

	n = ( high - low ) / step + 1;

	assert ( n > 0 );

	status = intlist_ensure_space( il, n );

	if ( status==INTLIST_OK ) {

		il->n = 0;

		/* ...fill intlist with range */
		if ( step > 0 ) {
			for ( i=low; i<high; i+=step ) {
				il->data[il->n] = i;
				il->n += 1;
			}
		}
		else {
			for ( i=low; i>high; i+=step ) {
				il->data[il->n] = i;
				il->n += 1;
			}
		}

	}

	return status;
}

static int
intcomp( const void *v1, const void *v2 )
{
	int *i1 = ( int * ) v1;
	int *i2 = ( int * ) v2;
	if ( *i1 < *i2 ) return -1;
	else if ( *i1 > *i2 ) return 1;
	return 0;
}

void
intlist_sort( intlist *il )
{
	assert( il );

	qsort( il->data, il->n, sizeof( int ), intcomp );
}

/* Returns random integer in the range [floor,ceil) */
static int
randomint( int floor, int ceil )
{
	int len = ceil - floor;
	return floor + rand() % len;
}

static void
swap( int *a, int *b )
{
	int tmp;
	tmp = *a;
	*a = *b;
	*b = tmp;
}

void
intlist_randomize( intlist *il )
{
	int i, j;

	assert( il );

	if ( il->n < 2 ) return;
	for ( i=0; i<il->n; ++i ) {
		j = randomint( i, il->n );
		if ( i==j ) continue;
		swap( &(il->data[i]), &(il->data[j]) );
	}
}

/* Returns INTLIST_OK/INTLIST_MEMERR */
int
intlist_copy( intlist *to, intlist *from )
{
	int i, status;

	assert( to );
	assert( from );

	status = intlist_ensure_space( to, from->n );

	if ( status==INTLIST_OK ) {

		to->n = from->n;

		for ( i=0; i<from->n; ++i )
			to->data[i] = from->data[i];

	}

	return status;
}

/* Returns pointer on success, NULL on error */
intlist *
intlist_dup( intlist *il )
{
	intlist *l;
	int status;

	assert( il );

	l = intlist_new();
	if ( l ) {
		status = intlist_copy( l, il );
		if ( status==INTLIST_MEMERR ) {
			intlist_delete( l );
			l = NULL;
		}
	}

	return l;
}

int
intlist_append( intlist *to, intlist *from )
{
	int i, status;

	assert( to );
	assert( from );

	status = intlist_ensure_space( to, to->n + from->n );

	if ( status == INTLIST_OK ) {

		for ( i=0; i<from->n; ++i )
			to->data[ to->n + i ] = from->data[ i ];

		to->n += from->n;
	}

	return status;
}

int
intlist_append_unique( intlist *to, intlist *from )
{
	int i, nsave, status = INTLIST_OK;

	assert( to );
	assert( from );

	nsave = to->n;
	for ( i=0; i<from->n; ++i ) {
		if ( intlist_find( to, from->data[i] )!=-1 ) continue;
		status = intlist_add( to, from->data[i] );
		if ( status==INTLIST_MEMERR ) {
			to->n = nsave;
		}
	}
	return status;
}

int
intlist_get( intlist *il, int pos )
{
	assert( il );
	assert( intlist_validn( il, pos ) );

	return il->data[pos];
}

/* intlist_set()
 *
 *   Returns INTLIST_OK
 */
int
intlist_set( intlist *il, int pos, int value )
{
	assert( il );
	assert( intlist_validn( il, pos ) );

	il->data[pos] = value;
	return INTLIST_OK;
}

float
intlist_median( intlist *il )
{
	intlist *tmp;
	float median;
	int m1, m2;

	assert( il );

	if ( il->n==0 ) return 0.0;

	tmp = intlist_dup( il );
	if ( !tmp ) return 0.0;

	intlist_sort( tmp );

	if ( tmp->n % 2 == 1 ) {
		median = intlist_get( tmp, tmp->n / 2 );
	} else {
		m1 = intlist_get( tmp, tmp->n / 2 );
		m2 = intlist_get( tmp, tmp->n / 2 - 1);
		median = ( m1 + m2 ) / 2.0;
	}

	intlist_delete( tmp );

	return median;
}

float
intlist_mean( intlist *il )
{
	float sum = 0.0;
	int i;

	assert( il );

	if ( il->n==0 ) return 0.0;

	for ( i=0; i<il->n; ++i )
		sum += intlist_get( il, i );

	return sum / il->n;
}
