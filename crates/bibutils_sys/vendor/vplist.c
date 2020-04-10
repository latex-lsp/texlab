/*
 * vplist.c
 *
 * Version: 1/9/2017
 *
 * Copyright (c) Chris Putnam 2011-2019
 *
 * Source code released under the GPL version 2
 *
 * Implements a simple managed array of pointers to void
 *
 */
#include <stdlib.h>
#include "vplist.h"

/* Do not use asserts in VPLIST_NOASSERT defined */
#ifdef VPLIST_NOASSERT
#define NDEBUG
#endif
#include <assert.h>

#define VPLIST_MINALLOC (20)

#define VPLIST_EXACT_SIZE  (0)
#define VPLIST_DOUBLE_SIZE (1)

void
vplist_init( vplist *vpl )
{
	assert( vpl );
	vpl->data = NULL;
	vpl->n = vpl->max = 0;
}

vplist *
vplist_new( void )
{
	vplist *vpl;
	vpl = ( vplist * ) malloc( sizeof( vplist ) );
	if ( vpl ) vplist_init( vpl );
	return vpl;
}

static inline int
vplist_alloc( vplist *vpl, vplist_index alloc )
{
	vpl->data = ( void ** ) malloc( sizeof( void * ) * alloc );
	if ( !vpl->data ) return VPLIST_MEMERR;

	vpl->max = alloc;
	vpl->n = 0;

	return VPLIST_OK;
}

static inline int
vplist_realloc( vplist *vpl, vplist_index alloc )
{
	void **more;

	more = ( void ** ) realloc( vpl->data, sizeof( void * ) * alloc );
	if ( !more ) return VPLIST_MEMERR;

	vpl->data = more;
	vpl->max  = alloc;

	return VPLIST_OK;
}

/* vplist_ensure_space( vpl, n, mode )
 *
 *    Makes sure that vplist can hold at least n members, allocating memory if required.
 *
 *    mode
 *       - Can either be VPLIST_DOUBLE_SIZE or VPLIST_EXACT_SIZE.
 *       - If VPLIST_EXACT_SIZE and current size < n, size will be exactly n.
 *       - If VPLIST_DOUBLE_SIZE and current size < n, size will be doubled (or VPLIST_MINALLOC 
 *         if the vplist is empty) or n, whichever is bigger.
 *
 *    Returns VPLIST_OK or VPLIST_MEMERR.
 */
static int
vplist_ensure_space( vplist *vpl, vplist_index n, unsigned char mode )
{
	vplist_index alloc = n;
	int status = VPLIST_OK;

	if ( vpl->max == 0 ) {
		if ( mode == VPLIST_DOUBLE_SIZE && alloc < VPLIST_MINALLOC ) alloc = VPLIST_MINALLOC;
		status = vplist_alloc( vpl, alloc );
	}

	else if ( vpl->max < n ) {
		if ( mode == VPLIST_DOUBLE_SIZE && alloc < 2 * vpl->max ) alloc = 2 * vpl->max;
		status = vplist_realloc( vpl, alloc );
	}

	return status;
}

int
vplist_copy( vplist *to, vplist *from )
{
	vplist_index i;
	int status;

	assert( to );
	assert( from );

	status = vplist_ensure_space( to, from->n, VPLIST_EXACT_SIZE );

	if ( status == VPLIST_OK ) {

		for ( i=0; i<from->n; ++i )
			to->data[i] = from->data[i];
		to->n = from->n;

	}

	return status;
}

int
vplist_fill( vplist *vpl, vplist_index n, void *v )
{
	vplist_index i;
	int status;

	assert( vpl );

	status = vplist_ensure_space( vpl, n, VPLIST_EXACT_SIZE );

	if ( status == VPLIST_OK ) {

		for ( i=0; i<n; ++i )
			vpl->data[i] = v;
		vpl->n = n;

	}

	return status;
}

int
vplist_add( vplist *vpl, void *v )
{
	int status;

	assert( vpl );

	status = vplist_ensure_space( vpl, vpl->n + 1, VPLIST_DOUBLE_SIZE );

	if ( status == VPLIST_OK ) {

		vpl->data[vpl->n] = v;
		vpl->n++;

	}

	return status;
}

int
vplist_insert_list( vplist *vpl, vplist_index pos, vplist *add )
{
	vplist_index i;
	int status;

	assert( vpl );
	assert( add );
	assert( pos <= vpl->n );

	/* nothing to do here */
	if ( add->n < 1 ) return VPLIST_OK;

	status = vplist_ensure_space( vpl, vpl->n + add->n, VPLIST_DOUBLE_SIZE );

	if ( status == VPLIST_OK ) {

		for ( i=vpl->n-1; i>=pos; --i )
			vpl->data[i+add->n] = vpl->data[i];

		for ( i=0; i<add->n; ++i )
			vpl->data[pos+i] = add->data[i];

		vpl->n += add->n;
	}

	return status;
}

int
vplist_append( vplist *vpl, vplist *add )
{
	vplist_index i;
	int status;

	assert( vpl );
	assert( add );

	status = vplist_ensure_space( vpl, vpl->n + add->n, VPLIST_DOUBLE_SIZE );

	if ( status == VPLIST_OK ) {

		for ( i=0; i<add->n; ++i )
			vpl->data[ vpl->n + i ] = add->data[i];

		vpl->n += add->n;

	}

	return status;
}

static void
vplist_freemembers( vplist *vpl, vplist_ptrfree vpf )
{
	vplist_index i;
	void *v;
	for ( i=0; i<vpl->n; ++i ) {
		v = vplist_get( vpl, i );
		if ( v ) (*vpf)( v );
	}
}

void
vplist_emptyfn( vplist *vpl, vplist_ptrfree vpf )
{
	assert( vpl );
	if ( vpf ) vplist_freemembers( vpl, vpf );
	vpl->n = 0;
}

void
vplist_empty( vplist *vpl )
{
	vplist_emptyfn( vpl, NULL );
}

void
vplist_freefn( vplist *vpl, vplist_ptrfree vpf )
{
	assert( vpl );
	if ( vpf ) vplist_freemembers( vpl, vpf );
	if ( vpl->data ) free( vpl->data );
	vplist_init( vpl );
}

void
vplist_free( vplist *vpl )
{
	vplist_freefn( vpl, NULL );
}

void
vplist_deletefn( vplist **vpl, vplist_ptrfree vpf )
{
	vplist_freefn( *vpl, vpf );
	free( *vpl );
	*vpl = NULL;
}

void
vplist_delete( vplist **vpl )
{
	vplist_deletefn( vpl, NULL );
}

static inline int
vplist_validindex( vplist *vpl, vplist_index n )
{
	if ( n < 0 || n >= vpl->n ) return 0;
	return 1;
}

void *
vplist_get( vplist *vpl, vplist_index n )
{
	assert( vpl );
	if ( !vplist_validindex( vpl, n ) ) return NULL;
	return vpl->data[ n ];
}

void
vplist_set( vplist *vpl, vplist_index n, void *v )
{
	assert( vpl );
	assert( vplist_validindex( vpl, n ) );
	vpl->data[ n ] = v;
}

int
vplist_find( vplist *vpl, void *v )
{
	vplist_index i;
	assert( vpl );
	for ( i=0; i<vpl->n; ++i )
		if ( vpl->data[i]==v ) return i;
	return -1;
}

void
vplist_swap( vplist *vpl, vplist_index n1, vplist_index n2 )
{
	void *tmp;

	assert( vpl );
	assert( vplist_validindex( vpl, n1 ) );
	assert( vplist_validindex( vpl, n2 ) );

	tmp           = vpl->data[n1];
	vpl->data[n1] = vpl->data[n2];
	vpl->data[n2] = tmp;
}

int
vplist_removefn( vplist *vpl, vplist_index n, vplist_ptrfree vpf )
{
	vplist_index i;

	assert( vpl );
	assert( vplist_validindex( vpl, n ) );

	if ( vpf ) (*vpf)( vplist_get( vpl, n ) );

	for ( i=n+1; i<vpl->n; ++i )
		vpl->data[ i-1 ] = vpl->data[ i ];
	vpl->n -= 1;

	return 1;
}

int
vplist_remove( vplist *vpl, vplist_index n )
{
	return vplist_removefn( vpl, n, NULL );
}

int
vplist_removevpfn( vplist *vpl, void *v, vplist_ptrfree vpf )
{
	vplist_index n;
	int count = 0;

	assert( vpl );

	do {
		n = vplist_find( vpl, v );
		if ( vplist_found( vpl, n ) ) {
			vplist_removefn( vpl, n, vpf );
			count++;
		}
	} while ( vplist_found( vpl, n ) );

	return count;
}

int
vplist_removevp( vplist *vpl, void *v )
{
	return vplist_removevpfn( vpl, v, NULL );
}

void
vplist_remove_rangefn( vplist *vpl, vplist_index start, vplist_index endplusone, vplist_ptrfree vpf )
{
	vplist_index i, n;

	assert( endplusone <= vpl->n );
	assert( endplusone > start );

	n = endplusone - start;
	if ( vpf ) {
		for ( i=start; i<endplusone; ++i )
			(*vpf)( vplist_get( vpl, i ) );
	}
	for ( i=endplusone; i<vpl->n; ++i ) {
		vpl->data[i-n] = vpl->data[i];
	}
	vpl->n -= n;
}

void
vplist_remove_range( vplist *vpl, vplist_index start, vplist_index endplusone )
{
	vplist_remove_rangefn( vpl, start, endplusone, NULL );
}
