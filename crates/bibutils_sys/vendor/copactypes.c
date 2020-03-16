/*
 * copactypes.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Program and source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <string.h>
#include "is_ws.h"
#include "fields.h"
#include "reftypes.h"

/* if no specific type can be identified */
static lookups generic[] = {
	{ "TI-", "TITLE" ,       TITLE,  LEVEL_MAIN },
	{ "AU-", "AUTHOR",       PERSON, LEVEL_MAIN },
	{ "MV-", "VOLUME",       SIMPLE, LEVEL_MAIN },
	{ "SE-", "TITLE",        TITLE,  LEVEL_HOST },
	{ "ED-", "EDITION",      SIMPLE, LEVEL_MAIN },
	{ "SC-", "SCALE",        SIMPLE, LEVEL_MAIN }, /* for maps */
	{ "PU-", "PUBLISHER",    SIMPLE, LEVEL_MAIN },
	{ "PY-", "DATE:YEAR",    SIMPLE, LEVEL_MAIN },
	{ "PD-", "DESCRIPTION",  SIMPLE, LEVEL_MAIN }, /* physical description */
	{ "DT-", "TYPE",         SIMPLE, LEVEL_MAIN },
	{ "LA-", "LANGUAGE",     SIMPLE, LEVEL_MAIN },
	{ "IS-", "SERIALNUMBER", SERIALNO, LEVEL_MAIN },
	{ "NT-", "NOTES",        NOTES,  LEVEL_MAIN },
	{ "KW-", "KEYWORD",      SIMPLE, LEVEL_MAIN },
	{ "UL-", "URL",          SIMPLE, LEVEL_MAIN },
	{ "HL-", "LOCATION",     SIMPLE, LEVEL_MAIN }
};

/* order is important....."Book" matches "Book" and "Book Section", hence
 * "Book Section must come first */

#define ORIG(a) ( &(a[0]) )
#define SIZE(a) ( sizeof( a ) / sizeof( lookups ) )
#define REFTYPE(a,b) { a, ORIG(b), SIZE(b) }

variants copac_all[] = {
	REFTYPE( "Generic", generic ),
};


int copac_nall = sizeof( copac_all ) / sizeof( variants );


