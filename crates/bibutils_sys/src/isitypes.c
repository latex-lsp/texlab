/*
 * isitypes.c
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

static lookups article[] = {
	{ "AU",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "AF",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "TI",     "TITLE",                 TITLE,    LEVEL_MAIN },
	{ "SO",     "TITLE",                 TITLE,    LEVEL_HOST }, /* full journal name */
	{ "JI",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* abbr journal name */
	{ "J9",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* 29char journal name */
	{ "PU",     "PUBLISHER",             SIMPLE,   LEVEL_HOST },
	{ "PI",     "ADDRESS",               SIMPLE,   LEVEL_HOST }, /* publisher city */
	{ "C1",     "ADDRESS",               SIMPLE,   LEVEL_MAIN }, /* author address */
	{ "PA",     "ADDRESS:PUBLISHER",     SIMPLE,   LEVEL_HOST }, /* publisher address */
	{ "RP",     "REPRINTADDRESS",        SIMPLE,   LEVEL_MAIN },
	{ "PY",     "PARTDATE:YEAR",         SIMPLE,   LEVEL_MAIN },
	{ "PD",     "PARTDATE:MONTH",        SIMPLE,   LEVEL_MAIN },
	{ "VL",     "VOLUME",                SIMPLE,   LEVEL_MAIN },
	{ "AR",     "ARTICLENUMBER",         SIMPLE,   LEVEL_MAIN }, /* AR=article number that Phys. Rev. B uses instead of page numbers */
	{ "BP",     "PAGES:START",           SIMPLE,   LEVEL_MAIN },
	{ "EP",     "PAGES:STOP",            SIMPLE,   LEVEL_MAIN },
	{ "PG",     "PAGES:TOTAL",           SIMPLE,   LEVEL_MAIN },
	{ "IS",     "ISSUE",                 SIMPLE,   LEVEL_MAIN },
	{ "SN",     "SERIALNUMBER",          SERIALNO, LEVEL_HOST },
	{ "AB",     "ABSTRACT",              SIMPLE,   LEVEL_MAIN },
	{ "NF",     "NOTES",                 NOTES,    LEVEL_MAIN },
	{ "DE",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* author keywords */
	{ "ID",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* new ISI keywords */
	{ "LA",     "LANGUAGE",              SIMPLE,   LEVEL_MAIN },
	{ "TC",     "TIMESCITED",            SIMPLE,   LEVEL_MAIN },
	{ "NR",     "NUMBERREFS",            SIMPLE,   LEVEL_MAIN },
	{ "CR",     "CITEDREFS",             SIMPLE,   LEVEL_MAIN },
	{ "PT",     " ",                     TYPE,     LEVEL_HOST },
	{ "DT",     "DOCUMENTTYPE",          TYPE,     LEVEL_MAIN },
	{ "GA",     "ISIDELIVERNUM",         SIMPLE,   LEVEL_MAIN }, /* ISI document delivery number */
	{ "UT",     "ISIREFNUM",             SIMPLE,   LEVEL_MAIN }, /* ISI unique article identifer */
	{ "DI",     "DOI",                   SIMPLE,   LEVEL_MAIN },
	{ " ",      "INTERNAL_TYPE|ARTICLE",           ALWAYS, LEVEL_MAIN },
	{ " ",      "ISSUANCE|continuing",             ALWAYS, LEVEL_HOST },
	{ " ",      "RESOURCE|text",                   ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:BIBUTILS|journal article",  ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:MARC|periodical",           ALWAYS, LEVEL_HOST },
	{ " ",      "GENRE:BIBUTILS|academic journal", ALWAYS, LEVEL_HOST }
};

static lookups book[] = {
	{ "AU",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "AF",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "TI",     "TITLE",                 TITLE,    LEVEL_MAIN },
	{ "SO",     "TITLE",                 TITLE,    LEVEL_HOST }, /* full journal name */
	{ "JI",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* abbr journal name */
	{ "J9",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* 29char journal name */
	{ "PU",     "PUBLISHER",             SIMPLE,   LEVEL_MAIN },
	{ "PI",     "ADDRESS",               SIMPLE,   LEVEL_MAIN }, /* publisher city */
	{ "C1",     "ADDRESS",               SIMPLE,   LEVEL_MAIN }, /* author address */
	{ "PA",     "ADDRESS:PUBLISHER",     SIMPLE,   LEVEL_MAIN }, /* publisher address */
	{ "RP",     "REPRINTADDRESS",        SIMPLE,   LEVEL_MAIN },
	{ "PY",     "DATE:YEAR",             SIMPLE,   LEVEL_MAIN },
	{ "PD",     "DATE:MONTH",            SIMPLE,   LEVEL_MAIN },
	{ "VL",     "VOLUME",                SIMPLE,   LEVEL_MAIN },
	{ "BP",     "PAGES:START",           SIMPLE,   LEVEL_MAIN },
	{ "EP",     "PAGES:STOP",            SIMPLE,   LEVEL_MAIN },
	{ "PG",     "PAGES:TOTAL",           SIMPLE,   LEVEL_MAIN },
	{ "IS",     "ISSUE",                 SIMPLE,   LEVEL_MAIN },
	{ "SN",     "SERIALNUMBER",          SERIALNO, LEVEL_HOST },
	{ "AB",     "ABSTRACT",              SIMPLE,   LEVEL_MAIN },
	{ "NF",     "NOTES",                 NOTES,    LEVEL_MAIN },
	{ "DE",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* author keywords */
	{ "ID",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* new ISI keywords */
	{ "LA",     "LANGUAGE",              SIMPLE,   LEVEL_MAIN },
	{ "TC",     "TIMESCITED",            SIMPLE,   LEVEL_MAIN },
	{ "NR",     "NUMBERREFS",            SIMPLE,   LEVEL_MAIN },
	{ "CR",     "CITEDREFS",             SIMPLE,   LEVEL_MAIN },
	{ "PT",     " ",                     TYPE,     LEVEL_MAIN },
	{ "DT",     "DOCUMENTTYPE",          TYPE,     LEVEL_MAIN },
	{ "GA",     "ISIDELIVERNUM",         SIMPLE,   LEVEL_MAIN }, /* ISI document delivery number */
	{ "UT",     "ISIREFNUM",             SIMPLE,   LEVEL_MAIN }, /* ISI unique article identifer */
	{ "PT",     " ",                     TYPE,     LEVEL_HOST },
	{ "DI",     "DOI",                   SIMPLE,   LEVEL_MAIN },
	{ " ",      "INTERNAL_TYPE|BOOK",              ALWAYS, LEVEL_MAIN },
	{ " ",      "ISSUANCE|monographic",            ALWAYS, LEVEL_MAIN },
	{ " ",      "RESOURCE|text",                   ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:MARC|book",                 ALWAYS, LEVEL_MAIN }
};

static lookups inbook[] = {
	{ "AU",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "AF",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "TI",     "TITLE",                 TITLE,    LEVEL_MAIN },
	{ "SO",     "TITLE",                 TITLE,    LEVEL_HOST }, /* full journal name */
	{ "JI",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* abbr journal name */
	{ "J9",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* 29char journal name */
	{ "PU",     "PUBLISHER",             SIMPLE,   LEVEL_HOST },
	{ "PI",     "ADDRESS",               SIMPLE,   LEVEL_HOST }, /* publisher city */
	{ "C1",     "ADDRESS",               SIMPLE,   LEVEL_MAIN }, /* author address */
	{ "PA",     "ADDRESS:PUBLISHER",     SIMPLE,   LEVEL_HOST }, /* publisher address */
	{ "RP",     "REPRINTADDRESS",        SIMPLE,   LEVEL_MAIN },
	{ "PY",     "PARTDATE:YEAR",         SIMPLE,   LEVEL_MAIN },
	{ "PD",     "PARTDATE:MONTH",        SIMPLE,   LEVEL_MAIN },
	{ "VL",     "VOLUME",                SIMPLE,   LEVEL_MAIN },
	{ "BP",     "PAGES:START",           SIMPLE,   LEVEL_MAIN },
	{ "EP",     "PAGES:STOP",            SIMPLE,   LEVEL_MAIN },
	{ "PG",     "PAGES:TOTAL",           SIMPLE,   LEVEL_HOST },
	{ "IS",     "ISSUE",                 SIMPLE,   LEVEL_MAIN },
	{ "SN",     "SERIALNUMBER",          SERIALNO, LEVEL_HOST },
	{ "AB",     "ABSTRACT",              SIMPLE,   LEVEL_MAIN },
	{ "NF",     "NOTES",                 NOTES,    LEVEL_MAIN },
	{ "DE",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* author keywords */
	{ "ID",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* new ISI keywords */
	{ "LA",     "LANGUAGE",              SIMPLE,   LEVEL_MAIN },
	{ "TC",     "TIMESCITED",            SIMPLE,   LEVEL_MAIN },
	{ "NR",     "NUMBERREFS",            SIMPLE,   LEVEL_MAIN },
	{ "CR",     "CITEDREFS",             SIMPLE,   LEVEL_MAIN },
	{ "PT",     " ",                     TYPE,     LEVEL_HOST },
	{ "DT",     "DOCUMENTTYPE",          TYPE,     LEVEL_MAIN },
	{ "GA",     "ISIDELIVERNUM",         SIMPLE,   LEVEL_MAIN }, /* ISI document delivery number */
	{ "UT",     "ISIREFNUM",             SIMPLE,   LEVEL_MAIN }, /* ISI unique article identifer */
	{ "DI",     "DOI",                   SIMPLE,   LEVEL_MAIN },
	{ " ",      "INTERNAL_TYPE|INBOOK",            ALWAYS, LEVEL_MAIN },
	{ " ",      "RESOURCE|text",                   ALWAYS, LEVEL_MAIN },
	{ " ",      "ISSUANCE|monographic",            ALWAYS, LEVEL_HOST },
        { " ",      "GENRE:BIBUTILS|book chapter",     ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:MARC|book",                 ALWAYS, LEVEL_HOST }
};

static lookups bookinseries[] = {
	{ "AU",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "AF",     "AUTHOR",                PERSON,   LEVEL_MAIN },
	{ "TI",     "TITLE",                 TITLE,    LEVEL_MAIN },
	{ "SO",     "TITLE",                 TITLE,    LEVEL_MAIN },
	{ "SE",     "TITLE",                 TITLE,    LEVEL_HOST },
	{ "BS",     "SUBTITLE",              TITLE,    LEVEL_HOST },
	{ "JI",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* abbr journal name */
	{ "J9",     "SHORTTITLE",            TITLE,    LEVEL_HOST }, /* 29char journal name */
	{ "PU",     "PUBLISHER",             SIMPLE,   LEVEL_HOST },
	{ "PI",     "ADDRESS",               SIMPLE,   LEVEL_HOST }, /* publisher city */
	{ "C1",     "ADDRESS",               SIMPLE,   LEVEL_MAIN }, /* author address */
	{ "PA",     "ADDRESS:PUBLISHER",     SIMPLE,   LEVEL_HOST }, /* publisher address */
	{ "RP",     "REPRINTADDRESS",        SIMPLE,   LEVEL_MAIN },
	{ "PY",     "PARTDATE:YEAR",         SIMPLE,   LEVEL_MAIN },
	{ "PD",     "PARTDATE:MONTH",        SIMPLE,   LEVEL_MAIN },
	{ "VL",     "VOLUME",                SIMPLE,   LEVEL_MAIN },
	{ "BP",     "PAGES:START",           SIMPLE,   LEVEL_MAIN },
	{ "EP",     "PAGES:STOP",            SIMPLE,   LEVEL_MAIN },
	{ "PG",     "PAGES:TOTAL",           SIMPLE,   LEVEL_MAIN },
	{ "IS",     "ISSUE",                 SIMPLE,   LEVEL_MAIN },
	{ "SN",     "SERIALNUMBER",          SERIALNO, LEVEL_HOST },
	{ "AB",     "ABSTRACT",              SIMPLE,   LEVEL_MAIN },
	{ "NF",     "NOTES",                 NOTES,    LEVEL_MAIN },
	{ "DE",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* author keywords */
	{ "ID",     "KEYWORD",               KEYWORD,  LEVEL_MAIN }, /* new ISI keywords */
	{ "LA",     "LANGUAGE",              SIMPLE,   LEVEL_MAIN },
	{ "TC",     "TIMESCITED",            SIMPLE,   LEVEL_MAIN },
	{ "NR",     "NUMBERREFS",            SIMPLE,   LEVEL_MAIN },
	{ "CR",     "CITEDREFS",             SIMPLE,   LEVEL_MAIN },
	{ "PT",     " ",                     TYPE,     LEVEL_HOST },
	{ "DT",     "DOCUMENTTYPE",          TYPE,     LEVEL_MAIN },
	{ "GA",     "ISIDELIVERNUM",         SIMPLE,   LEVEL_MAIN }, /* ISI document delivery number */
	{ "UT",     "ISIREFNUM",             SIMPLE,   LEVEL_MAIN }, /* ISI unique article identifer */
	{ "DI",     "DOI",                   SIMPLE,   LEVEL_MAIN },
	{ " ",      "INTERNAL_TYPE|INCOLLECTION",      ALWAYS, LEVEL_MAIN },
	{ " ",      "ISSUANCE|monographic",            ALWAYS, LEVEL_HOST },
	{ " ",      "RESOURCE|text",                   ALWAYS, LEVEL_MAIN },
	{ " ",      "GENRE:BIBUTILS|collection",       ALWAYS, LEVEL_MAIN }
};

#define ORIG(a) ( &(a[0]) )
#define SIZE(a) ( sizeof( a ) / sizeof( lookups ) )
#define REFTYPE(a,b) { a, ORIG(b), SIZE(b) }

variants isi_all[] = {
	REFTYPE( "Journal", article ),
	REFTYPE( "J", article ),
	REFTYPE( "Book", book ),
	REFTYPE( "B", book ),
	REFTYPE( "Chapter", inbook ),
	REFTYPE( "S", bookinseries ),
};

int isi_nall = sizeof( isi_all ) / sizeof( variants );

