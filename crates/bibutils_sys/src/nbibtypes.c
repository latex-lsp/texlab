/*
 * nbibtypes.c
 *
 * Copyright (c) Chris Putnam 2016-2019
 *
 * Program and source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <string.h>
#include "is_ws.h"
#include "fields.h"
#include "reftypes.h"

static lookups article[] = {
	{ "PMID",   "PMID",               SIMPLE, LEVEL_MAIN },
	{ "OWN",    "",                   SKIP,   LEVEL_MAIN },
	{ "STAT",   "",                   SKIP,   LEVEL_MAIN },
	{ "DA",     "",                   SKIP,   LEVEL_MAIN },
	{ "DCOM",   "",                   SKIP,   LEVEL_MAIN },
	{ "LR",     "",                   SKIP,   LEVEL_MAIN },
	{ "IS",     "",                   SKIP,   LEVEL_MAIN },
	{ "VI",     "VOLUME",             SIMPLE, LEVEL_MAIN },
	{ "IP",     "ISSUE",              SIMPLE, LEVEL_MAIN },
	{ "DP",     "",                   DATE,   LEVEL_MAIN }, /* date published? */
	{ "TI",     "TITLE",              TITLE,  LEVEL_MAIN },
	{ "PG",     "PAGES",              PAGES,  LEVEL_MAIN },
	{ "LID",    "DOI",                DOI,    LEVEL_MAIN }, /* linking ID? -- can be DOI/PII */
	{ "AB",     "ABSTRACT",           SIMPLE, LEVEL_MAIN },
	{ "FAU",    "AUTHOR",             PERSON, LEVEL_MAIN },
	{ "AU",     "",                   SKIP,   LEVEL_MAIN },
	{ "AD",     "ADDRESS:AUTHOR",     SIMPLE, LEVEL_MAIN },
	{ "LA",     "LANGUAGE",           SIMPLE, LEVEL_MAIN },
	{ "GR",     "",                   SKIP,   LEVEL_MAIN },
	{ "DEP",    "",                   SKIP,   LEVEL_MAIN }, /* a date */
	{ "PL",     "ADDRESS",            SIMPLE, LEVEL_MAIN }, /* Publisher location */
	{ "TA",     "SHORTTITLE",         SIMPLE, LEVEL_HOST }, /* Journal title abbreviation */
	{ "JT",     "TITLE",              SIMPLE, LEVEL_HOST }, /* Journal title */
	{ "JID",    "",                   SKIP,   LEVEL_HOST }, /* Journal ID? */
	{ "SB",     "",                   SKIP,   LEVEL_MAIN },
	{ "MH",     "KEYWORD",            SIMPLE, LEVEL_MAIN },
	{ "OT",     "KEYWORD",            SIMPLE, LEVEL_MAIN },
	{ "PMC",    "PMC",                SIMPLE, LEVEL_MAIN },
	{ "OID",    "",                   SKIP,   LEVEL_MAIN },
	{ "EDAT",   "",                   SKIP,   LEVEL_MAIN },
	{ "MHDA",   "",                   SKIP,   LEVEL_MAIN },
	{ "CRDT",   "",                   SKIP,   LEVEL_MAIN },
	{ "PHST",   "",                   SKIP,   LEVEL_MAIN }, /* Publication history? */
	{ "AID",    "DOI",                DOI,    LEVEL_MAIN }, /* Article ID? -- can be DOI/PII */
	{ "PST",    "",                   SKIP,   LEVEL_MAIN },
	{ "SO",     "",                   SKIP,   LEVEL_MAIN },
	{ " ",      "INTERNAL_TYPE|ARTICLE",   ALWAYS, LEVEL_MAIN },
	{ " ",      "ISSUANCE|continuing",     ALWAYS, LEVEL_HOST },
	{ " ",      "RESOURCE|text",           ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:BIBUTILS|journal article",  ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:MARC|periodical",        ALWAYS, LEVEL_HOST },
	{ " ",      "GENRE:BIBUTILS|academic journal", ALWAYS, LEVEL_HOST }
};

#define ORIG(a) ( &(a[0]) )
#define SIZE(a) ( sizeof( a ) / sizeof( lookups ) )
#define REFTYPE(a,b) { a, ORIG(b), SIZE(b) }

variants nbib_all[] = {
	REFTYPE( "Journal article", article ),
	REFTYPE( "News",            article ),
};

int nbib_nall = sizeof( nbib_all ) / sizeof( variants );

