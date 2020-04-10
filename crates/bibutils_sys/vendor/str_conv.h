/*
 * str_conv.h
 *
 * Copyright (c) Chris Putnam 1999-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef STR_CONV_H
#define STR_CONV_H

#define STR_CONV_XMLOUT_FALSE    (0)
#define STR_CONV_XMLOUT_TRUE     (1)
#define STR_CONV_XMLOUT_ENTITIES (3)

#include "str.h"

extern int str_convert( str *s,
		int charsetin, int latexin, int utf8in, int xmlin, 
		int charsetout, int latexout, int utf8out, int xmlout );

#endif

