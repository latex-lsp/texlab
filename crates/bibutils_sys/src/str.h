/*
 * str.h
 *
 * Version: 2018-09-21
 *
 * Copyright (c) Chris Putnam 1999-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef STR_H
#define STR_H

#define STR_OK (0)
#define STR_MEMERR (-1)

#include <stdio.h>

typedef struct str {
	char *data;
	unsigned long dim;
	unsigned long len;
#ifndef STR_SMALL
	int status;
#endif
}  str;

str *  str_new         ( void );
void   str_delete      ( str *s );

void   str_init        ( str *s );
void   str_initstr     ( str *s, str *from );
void   str_initstrc    ( str *s, const char *initstr );
void   str_initstrsc   ( str *s, ... );
void   str_empty       ( str *s );
void   str_free        ( str *s );

void   strs_init       ( str *s, ... );
void   strs_empty      ( str *s, ... );
void   strs_free       ( str *s, ... );

str*   str_strdup ( str *s );
str*   str_strdupc( const char *p );

void   str_strcat ( str *s, str *from );
void   str_strcatc( str *s, const char *from );

void   str_strcpy ( str *s, str *from );
void   str_strcpyc( str *s, const char *from );

int    str_strcmp ( const str *s, const str *t );
int    str_strcmpc( const str *s, const char *t );

int    str_strncmp ( const str *s, const str *t,  size_t n );
int    str_strncmpc( const str *s, const char *t, size_t n );

int    str_strcasecmp ( const str *s, const str *t );
int    str_strcasecmpc( const str *s, const char *t );

char * str_strstr ( const str *s, const str *t );
char * str_strstrc( const str *s, const char *t );

void str_prepend     ( str *s, const char *addstr );
void str_mergestrs   ( str *s, ... );

void str_addchar     ( str *s, char newchar );
void str_reverse     ( str *s );
const char *str_addutf8    ( str *s, const char *p );
void str_segcat      ( str *s, char *startat, char *endat );
const char *str_cpytodelim  ( str *s, const char *p, const char *delim, unsigned char finalstep );
const char *str_cattodelim  ( str *s, const char *p, const char *delim, unsigned char finalstep );
void str_prepend     ( str *s, const char *addstr );
void str_segcpy      ( str *s, char *startat, char *endat );
void str_segdel      ( str *s, char *startat, char *endat );
void str_indxcpy     ( str *s, char *p, unsigned long start, unsigned long stop );
void str_indxcat     ( str *s, char *p, unsigned long start, unsigned long stop );
void str_fprintf     ( FILE *fp, str *s );
int  str_fget        ( FILE *fp, char *buf, int bufsize, int *pbufpos,
                          str *outs );
char * str_cstr      ( str *s );
char str_char        ( str *s, unsigned long n );
char str_revchar     ( str *s, unsigned long n );
int  str_fgetline    ( str *s, FILE *fp );
int  str_findreplace ( str *s, const char *find, const char *replace );
void str_toupper     ( str *s );
void str_tolower     ( str *s );
void str_trimstartingws( str *s );
void str_trimendingws( str *s );
void str_swapstrings ( str *s1, str *s2 );
void str_stripws     ( str *s );

int  str_match_first ( str *s, char ch );
int  str_match_end   ( str *s, char ch );
void str_trimbegin   ( str *s, unsigned long n );
void str_trimend     ( str *s, unsigned long n );

void str_pad         ( str *s, unsigned long len, char ch );
void str_copyposlen  ( str *s, str *in, unsigned long pos, unsigned long len );

void str_makepath    ( str *path, const char *dirname, const char *filename, char sep );

void str_fill        ( str *s, unsigned long n, char fillchar );

int  str_is_mixedcase( str *s );
int  str_is_lowercase( str *s );
int  str_is_uppercase( str *s );

int  str_memerr( str *s );

unsigned long str_strlen( str *s );

int  str_has_value( str *s );
int  str_is_empty( str *s );


/* #define STR_PARANOIA
 *
 * set to clear memory before it is freed or reallocated
 * note that this is slower...may be important if string
 * contains sensitive information
 */

/* #define STR_NOASSERT
 *
 * set to turn off the use of asserts (and associated call to exit)
 * in str functions...useful for library construction for
 * Linux distributions that don't want libraries calling exit, but
 * not useful during code development
 */

/* #define STR_SMALL
 *
 * set to make the smallest possible struct str, but will get
 * exit( EXIT_FAILURE ) upon memory failures
 */

#endif

