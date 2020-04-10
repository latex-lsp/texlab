/*
 * utf8.h
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef UTF8_H
#define UTF8_H

#include <stdio.h>

int          utf8_encode( unsigned int value, unsigned char out[6] );
void         utf8_encode_str( unsigned int value, char outstr[7] );
unsigned int utf8_decode( const char *s, unsigned int *pi );
void         utf8_writebom( FILE *outptr );
int          utf8_is_bom( const char *p );
int          utf8_is_emdash( const char *p );
int          utf8_is_endash( const char *p );

#endif
