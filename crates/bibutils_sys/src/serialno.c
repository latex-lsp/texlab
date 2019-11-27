/*
 * serialno.c
 *
 * Copyright (c) Chris Putnam 2005-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <string.h>
#include "serialno.h"

int
addsn( fields *info, char *buf, int level )
{
	int ndigits, issn=0, isbn=0, isbn10=0, isbn13=0, status;
	char *p = buf, *tag;

	if ( !strncasecmp( p, "ISSN", 4 ) ) issn=1;
	else if ( !strncasecmp( p, "ISBN", 4 ) ) isbn=1;

	if ( isbn ) {
		ndigits = 0;
		while ( *p && !(ndigits && (*p==';'||*p==':')) ) {
			if ( ( *p>='0' && *p<='9' ) || *p=='x' || *p=='X' )
				ndigits++;
			p++;
		}
		if ( ndigits==13 ) isbn13 = 1;
		else /* ( ndigits==10) */ isbn10 = 1;
	}

	if ( !issn && !isbn ) {
		/* a lot have semicolons between multiple ISBN's for
		   paperbacks and hardbacks with different numbers */
		ndigits = 0;
		while ( *p && !(ndigits && (*p==';'||*p==':')) ) {
			if ( ( *p>='0' && *p<='9' ) || *p=='x' || *p=='X' )
				ndigits++;
			p++;
		}
		if ( ndigits==8 ) issn = 1;
		else if ( ndigits==10 ) isbn10 = 1;
		else if ( ndigits==13 ) isbn13 = 1;
	}
	
	if ( issn ) tag = "ISSN";
	else if ( isbn10 ) tag = "ISBN";
	else if ( isbn13 ) tag = "ISBN13";
	else tag = "SERIALNUMBER";

	status = fields_add( info, tag, buf, level );

	if ( status==FIELDS_OK ) return 1;
	else return 0;
}
