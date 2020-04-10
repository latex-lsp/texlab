/*
 * is_ws.h
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef IS_WS_H
#define IS_WS_H

int is_ws( const char ch );
const char *skip_ws( const char *p );
const char *skip_notws( const char *p );
const char *skip_line( const char *p );

#endif

