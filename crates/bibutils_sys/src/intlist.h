/*
 * intlist.h
 *
 * Copyright (c) Chris Putnam 2007-2019
 *
 * Version 01/12/2017
 *
 * Source code released under the GPL version 2
 *
 */

#ifndef INTLIST_H
#define INTLIST_H

#define INTLIST_OK            (0)
#define INTLIST_MEMERR        (-1)
#define INTLIST_VALUE_MISSING (-2)

typedef struct intlist {
	int n, max;
	int *data;
} intlist;

void      intlist_init( intlist *il );
int       intlist_init_fill( intlist *il, int n, int value );
int       intlist_init_range( intlist *il, int low, int high, int step );
intlist * intlist_new( void );
intlist * intlist_new_fill( int n, int value );
intlist * intlist_new_range( int low, int high, int step );
void      intlist_delete( intlist *il );
void      intlist_sort( intlist *il );
void      intlist_randomize( intlist *il );
int       intlist_add( intlist *il, int value );
int       intlist_add_unique( intlist *il, int value );
int       intlist_fill( intlist *il, int n, int value );
int       intlist_fill_range( intlist *il, int low, int high, int step );
int       intlist_find( intlist *il, int searchvalue );
int       intlist_find_or_add( intlist *il, int searchvalue );
void      intlist_empty( intlist *il );
void      intlist_free( intlist *il );
int       intlist_copy( intlist *to, intlist *from );
intlist * intlist_dup( intlist *from );
int       intlist_get( intlist *il, int pos );
int       intlist_set( intlist *il, int pos, int value );
int       intlist_remove( intlist *il, int searchvalue );
int       intlist_remove_pos( intlist *il, int pos );
int       intlist_append( intlist *to, intlist *from );
int       intlist_append_unique( intlist *to, intlist *from );
float     intlist_median( intlist *il );
float     intlist_mean( intlist *il );

#endif
