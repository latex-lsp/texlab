/*
 * reftypes.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <string.h>
#include "is_ws.h"
#include "fields.h"
#include "reftypes.h"

int
get_reftype( const char *p, long refnum, char *progname, variants *all, int nall, char *tag, int *is_default, int chattiness )
{
	int i;

	p = skip_ws( p );

	*is_default = 0;

	for ( i=0; i<nall; ++i ) {
		if ( !strncasecmp( all[i].type, p, strlen(all[i].type) ) ) 
			return i;
	}

	*is_default = 1;

	if ( chattiness==REFTYPE_CHATTY ) {
		if ( progname ) fprintf( stderr, "%s: ", progname );
		fprintf( stderr, "Did not recognize type '%s' of refnum %ld (%s).\n"
			"\tDefaulting to %s.\n", p, refnum, tag, all[0].type );
	}

	return 0;
}

int
process_findoldtag( const char *oldtag, int reftype, variants all[], int nall )
{
        variants *v;
        int i;

        v = &(all[reftype]);
        for ( i=0; i<v->ntags; ++i ) {
                if ( !strcasecmp( (v->tags[i]).oldstr, oldtag ) )
                        return i;
	}
        return -1;
}

/* translate_oldtag()
 */
int
translate_oldtag( const char *oldtag, int reftype, variants all[], int nall,
		int *processingtype, int *level, char **newtag )
{
	int n;

	n = process_findoldtag( oldtag, reftype, all, nall );
	if ( n!=-1 ) {
		*processingtype = ((all[reftype]).tags[n]).processingtype;
		*level          = ((all[reftype]).tags[n]).level;
		*newtag         = ((all[reftype]).tags[n]).newstr;
		return 1;
	}

	return 0;
}
