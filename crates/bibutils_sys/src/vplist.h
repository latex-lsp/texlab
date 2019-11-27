/*
 * vplist.h
 *
 * generic container to hold a list of pointers to void
 *
 * Version: 1/9/2017
 *
 * Copyright (c) Chris Putnam 2011-2019
 *
 * Source code released under the GPL version 2
 *
 */

#ifndef VPLIST_H
#define VPLIST_H

#define VPLIST_MEMERR (-1)
#define VPLIST_OK     (0)

typedef int vplist_index;

typedef struct vplist {
	vplist_index n, max;
	void **data;
} vplist;

#define vplist_found( vpl, n ) ( n!=-1 )
#define vplist_notfound( vpl, n ) ( n==-1 )

typedef void (*vplist_ptrfree)(void*);

vplist *     vplist_new( void );

void   vplist_init          ( vplist *vpl );
int    vplist_add           ( vplist *vpl, void *v );
int    vplist_fill          ( vplist *vpl, vplist_index n, void *v );
int    vplist_copy          ( vplist *to,  vplist *from );
int    vplist_append        ( vplist *vpl, vplist *add );
int    vplist_insert_list   ( vplist *vpl, vplist_index pos, vplist *add );
void * vplist_get           ( vplist *vpl, vplist_index n );
void   vplist_set           ( vplist *vpl, vplist_index n, void *v );
void   vplist_swap          ( vplist *vpl, vplist_index n1, vplist_index n2 );
int    vplist_remove        ( vplist *vpl, vplist_index n );
int    vplist_removefn      ( vplist *vpl, vplist_index n, vplist_ptrfree vpf );
int    vplist_removevp      ( vplist *vpl, void *v );
int    vplist_removevpfn    ( vplist *vpl, void *v, vplist_ptrfree vpf );
void   vplist_remove_rangefn( vplist *vpl, vplist_index start, vplist_index endplusone, vplist_ptrfree vpf );
void   vplist_remove_range  ( vplist *vpl, vplist_index start, vplist_index endplusone );

vplist_index vplist_find( vplist *vpl, void *v );

/*
 * vplist_empty does not free space
 *
 * if members require their own free calls, then call vplist_emptyfn()
 *
 * void
 * member_free( void *v )
 * {
 *     member *m = ( member * ) v;
 *     member_free( m );
 *     free( m );
 * }
 * vplist_emptyfn( &vpl, member_free );
 *
 * if members are simply allocated with malloc(), then use free()
 *
 * vplist_emptyfn( &vpl, free );
 */
void   vplist_empty  ( vplist *vpl );
void   vplist_emptyfn( vplist *vpl, vplist_ptrfree fn );
/*
 * vplist_free frees the space for the data array of void * elements.
 *
 * if members require their own free calls, then call vplist_freefn()
 */
void vplist_free  ( vplist *vpl );
void vplist_freefn( vplist *vpl, vplist_ptrfree fn );
/*
 * vplist_delete does vplist_free and deallocates the struct
 * vplist * and replaces with NULL.
 */
void vplist_delete  ( vplist **vpl );
void vplist_deletefn( vplist **vpl, vplist_ptrfree fn );

#endif
