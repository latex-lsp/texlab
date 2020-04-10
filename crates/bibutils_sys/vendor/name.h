/*
 * name.h
 *
 * mangle names w/ and w/o commas
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef NAME_H
#define NAME_H

#include "str.h"
#include "slist.h"
#include "fields.h"

int  name_add( fields *info, const char *tag, const char *q, int level, slist *asis, slist *corps );
void name_build_withcomma( str *s, const char *p );
int  name_parse( str *outname, str *inname, slist *asis, slist *corps );
int  name_addsingleelement( fields *info, const char *tag, const char *name, int level, int corp );
int  name_addmultielement( fields *info, const char *tag, slist *tokens, int begin, int end, int level );
int  name_findetal( slist *tokens );

#endif

