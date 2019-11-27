/*
 * ristypes.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fields.h"
#include "reftypes.h"
	
static lookups generic[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author -- Series editors */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Name of Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE, LEVEL_MAIN },   /* File Attachments (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Resarch Notes -> Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated? */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
};

static lookups article[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author -- Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_HOST },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_HOST },   /* Publisher */
	{ "PY", "PARTDATE:YEAR",DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_HOST },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "PARTDATE:YEAR",DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "PARTDATE:MONTH",SIMPLE, LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|ARTICLE",           ALWAYS, LEVEL_MAIN },
	{ "  ", "ISSUANCE|continuing",             ALWAYS, LEVEL_HOST },
	{ "  ", "RESOURCE|text",                   ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:BIBUTILS|journal article",  ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|periodical",           ALWAYS, LEVEL_HOST },
	{ "  ", "GENRE:BIBUTILS|academic journal", ALWAYS, LEVEL_HOST }
};

/* magazine article */
static lookups magarticle[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editors */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_HOST },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_HOST },   /* Publisher */
	{ "PY", "PARTDATE:YEAR",DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_HOST },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "PARTDATE:YEAR",DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "PARTDATE:MONTH",SIMPLE, LEVEL_MAIN },   /* Access Date */
	{ "  ", "ISSUANCE|continuing",     ALWAYS, LEVEL_HOST },
	{ "  ", "RESOURCE|text",           ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|periodical",   ALWAYS, LEVEL_HOST },
	{ "  ", "GENRE:BIBUTILS|magazine", ALWAYS, LEVEL_HOST }
};

static lookups newsarticle[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_HOST },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_HOST },   /* Publisher */
	{ "PY", "PARTDATE:YEAR",DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_HOST },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "PARTDATE:YEAR",DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "PARTDATE:MONTH",SIMPLE, LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|NEWSARTICLE", ALWAYS, LEVEL_MAIN },
	{ "  ", "ISSUANCE|continuing",       ALWAYS, LEVEL_HOST },
	{ "  ", "RESOURCE|text",             ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|newspaper",           ALWAYS, LEVEL_HOST }
};

static lookups book[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_HOST },   /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title -- here abbreviated title for series*/
	{ "T3", "TITLE",        SIMPLE,  LEVEL_HOST },   /* 'Tertiary' Title -- series title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "ISSUANCE|monographic",   ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|book",             ALWAYS, LEVEL_MAIN },
	{ "  ", "RESOURCE|text",          ALWAYS, LEVEL_MAIN }
};

static lookups inbook[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_HOST },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_HOST },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_HOST },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_HOST },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_HOST },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_HOST },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_HOST },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_SERIES }, /* 'Secondary' Title -- here abbreviated title for series*/
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_HOST },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_HOST },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_HOST },   /* Access Date */
	{ "  ", "GENRE:BIBUTILS|book chapter",   ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|book",           ALWAYS, LEVEL_HOST },
	{ "  ", "ISSUANCE|monographic", ALWAYS, LEVEL_HOST },
	{ "  ", "RESOURCE|text",        ALWAYS, LEVEL_MAIN }
};

static lookups conference[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR:ASIS",  SIMPLE,  LEVEL_HOST },   /* 'Secondary' Author - Name of conference */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_HOST },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_HOST },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Book Title */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_SERIES },   /* 'Secondary' Title - Abbreviated series TItle*/
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title  - Series Title*/
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_HOST },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_HOST },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|CONFERENCE",     ALWAYS, LEVEL_MAIN },
	{ "  ", "RESOURCE|text",                ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|conference publication", ALWAYS, LEVEL_HOST }
};

static lookups statute[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|STATUTE", ALWAYS, LEVEL_MAIN },
	{ "  ", "RESOURCE|text",         ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|legislation",     ALWAYS, LEVEL_MAIN }
};

static lookups hearing[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|HEARING", ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:BIBUTILS|hearing",         ALWAYS, LEVEL_MAIN }
};

static lookups cases[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|CASE",              ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|legal case and case notes", ALWAYS, LEVEL_MAIN }
};

static lookups communication[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "ADDRESSEE",    PERSON,  LEVEL_MAIN },   /* SPECIAL */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "GENRE:UKNOWN", GENRE,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|ARTICLE",          ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:BIBUTILS|communication",   ALWAYS, LEVEL_MAIN }
};

static lookups thesis[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        SIMPLE,  LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|THESIS",  ALWAYS,  LEVEL_MAIN },
	{ "  ", "RESOURCE|text",         ALWAYS,  LEVEL_MAIN },
	{ "  ", "GENRE:MARC|thesis",          ALWAYS,  LEVEL_MAIN },
};

static lookups report[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        SIMPLE,  LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "INTERNAL_TYPE|REPORT",        ALWAYS, LEVEL_MAIN },
	{ "  ", "RESOURCE|text",               ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|technical report", ALWAYS, LEVEL_MAIN }
};

static lookups abstract[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        SIMPLE,  LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "GENRE:MARC|abstract or summary", ALWAYS,  LEVEL_MAIN }
};

static lookups program[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        SIMPLE,  LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "RESOURCE|software, multimedia", ALWAYS, LEVEL_MAIN }
};

static lookups patent[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ "  ", "RESOURCE|text", ALWAYS, LEVEL_MAIN },
	{ "  ", "GENRE:MARC|patent",  ALWAYS, LEVEL_MAIN }
};

static lookups electric[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ " ",  "RESOURCE|software, multimedia", ALWAYS, LEVEL_MAIN },
	{ " ",  "GENRE:MARC|electronic",              ALWAYS, LEVEL_MAIN },
};

static lookups pamphlet[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ " ",  "RESOURCE|text",  ALWAYS, LEVEL_MAIN },
	{ " ",  "GENRE:BIBUTILS|pamphlet", ALWAYS, LEVEL_MAIN },
};

static lookups map[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Name of Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE, LEVEL_MAIN },   /* File Attachments (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Number? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Resarch Notes -> Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated? */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ " ",  "RESOURCE|cartographic", ALWAYS, LEVEL_MAIN },
	{ " ",  "GENRE:MARC|map",        ALWAYS, LEVEL_MAIN }
};

static lookups unpublished[] = {
	{ "A1", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "A2", "AUTHOR",       PERSON,  LEVEL_HOST },   /* 'Secondary' Author */
	{ "A3", "EDITOR",       PERSON,  LEVEL_SERIES }, /* 'Tertiary' Author - Series editor */
	{ "A4", "AUTHOR",       PERSON,  LEVEL_SERIES }, /* 'Subsidiary' Author */
	{ "AB", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Abstract */
	{ "AD", "ADDRESS:AUTHOR",SIMPLE,  LEVEL_MAIN },   /* Author Address */
	{ "AU", "AUTHOR",       PERSON,  LEVEL_MAIN },   /* Author */
	{ "BT", "TITLE",        SIMPLE,  LEVEL_MAIN },   /* Book Title - Deprecated? */
	{ "C1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C6", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C7", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "C8", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'Custom' - put in "notes" */
	{ "CA", "CAPTION",      SIMPLE,  LEVEL_MAIN },   /* Caption */
	{ "CN", "CALLNUMBER",   SIMPLE,  LEVEL_MAIN },   /* Call Number */
	{ "CP", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CT", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "CY", "ADDRESS",      SIMPLE,  LEVEL_MAIN },   /* Place Published */
	{ "DA", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Date */
	{ "DB", "DATABASE",     SIMPLE,  LEVEL_MAIN },   /* Database */
	{ "DI", "DOI",          DOI,     LEVEL_MAIN },   /* Deprecated? */
	{ "DO", "DOI",          DOI,     LEVEL_MAIN },   /* DOI */
	{ "DP", "DATABASEPROV", SIMPLE,  LEVEL_MAIN },   /* Database Provider */
	{ "ED", "EDITOR",       PERSON,  LEVEL_MAIN },   /* Deprecated? */
	{ "EP", "PAGES:STOP",   SIMPLE,  LEVEL_MAIN },   /* End Page */
	{ "ET", "EDITION",      SIMPLE,  LEVEL_MAIN },   /* Edition */
	{ "ID", "REFNUM",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "IS", "ISSUE",        SIMPLE,  LEVEL_MAIN },   /* Number */
	{ "J1", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "J2", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Alternate Title, abbreviated book or journal */
	{ "JA", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JF", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "JO", "TITLE",        SIMPLE,  LEVEL_HOST },   /* Deprecated? */
	{ "KW", "KEYWORD",      SIMPLE,  LEVEL_MAIN },   /* Keywords */
	{ "L1", "FILEATTACH",   LINKEDFILE,  LEVEL_MAIN },   /* File Attachment (local, not URL) */
	{ "L4", "FIGATTACH",    LINKEDFILE,  LEVEL_MAIN },   /* Figure Attachment (local, not URL) */
	{ "LA", "LANGUAGE",     SIMPLE,  LEVEL_MAIN },   /* Language */
	{ "LB", "LABEL",        SIMPLE,  LEVEL_MAIN },   /* Label */
	{ "M1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M2", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Deprecated? */
	{ "M3", "NOTES",        NOTES,   LEVEL_MAIN },   /* Misc or Type of Work? */
	{ "N1", "NOTES",        NOTES,   LEVEL_MAIN },   /* Notes */
	{ "N2", "ABSTRACT",     SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "NV", "NUMVOLUMES",   SIMPLE,  LEVEL_MAIN },   /* Number of Volumes */
	{ "OP", "ORIGPUB",      SIMPLE,  LEVEL_MAIN },   /* Original Publication */
	{ "PB", "PUBLISHER",    SIMPLE,  LEVEL_MAIN },   /* Publisher */
	{ "PY", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Year */
	{ "RI", "REVIEWEDITEM", SIMPLE,  LEVEL_MAIN },   /* Reviewed Item */
	{ "RN", "NOTES",        NOTES,   LEVEL_MAIN },   /* Research Notes */
	{ "RP", "REPRINTSTATUS",SIMPLE,  LEVEL_MAIN },   /* Reprint Edition */
	{ "SE", "SECTION",      SIMPLE,  LEVEL_MAIN },   /* Section */
	{ "SN", "SERIALNUMBER", SERIALNO,LEVEL_MAIN },   /* ISBN/ISSN */
	{ "SP", "PAGES:START",  SIMPLE,  LEVEL_MAIN },   /* Start Page */
	{ "ST", "SHORTTITLE",   SIMPLE,  LEVEL_MAIN },   /* Short Title */
	{ "T1", "TITLE",        TITLE,   LEVEL_MAIN },   /* Deprecated? */
	{ "T2", "SHORTTITLE",   SIMPLE,  LEVEL_HOST },   /* 'Secondary' Title */
	{ "T3", "TITLE",        SIMPLE,  LEVEL_SERIES }, /* 'Tertiary' Title */
	{ "TI", "TITLE",        TITLE,   LEVEL_MAIN },   /* Title */
	{ "TT", "TRANSTITLE",   TITLE,   LEVEL_MAIN },   /* Translated Title */
	{ "U1", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U2", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U3", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U4", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "U5", "NOTES",        NOTES,   LEVEL_MAIN },   /* 'User' - Deprecated? */
	{ "UR", "URL",          URL,     LEVEL_MAIN },   /* URL */
	{ "VL", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Volume */
	{ "VO", "VOLUME",       SIMPLE,  LEVEL_MAIN },   /* Deprecated? */
	{ "Y1", "DATE:YEAR",    DATE,    LEVEL_MAIN },   /* Deprecated */
	{ "Y2", "DATE:MONTH",   SIMPLE,  LEVEL_MAIN },   /* Access Date */
	{ " ",  "RESOURCE|text",              ALWAYS, LEVEL_MAIN },
	{ " ",  "GENRE:BIBUTILS|unpublished", ALWAYS, LEVEL_MAIN }
};

#define ORIG(a) ( &(a[0]) )
#define SIZE(a) ( sizeof( a ) / sizeof( lookups ) )
#define REFTYPE(a,b) { a, ORIG(b), SIZE(b) }

variants ris_all[] = {
	REFTYPE( "STD", generic ),
	REFTYPE( "GEN", generic ),
	REFTYPE( "JOUR", article ),
	REFTYPE( "MGZN", magarticle ),
	REFTYPE( "BOOK", book ),
	REFTYPE( "CHAP", inbook ),
	REFTYPE( "CONF", conference ),
	REFTYPE( "STAT", statute ),
	REFTYPE( "HEAR", hearing ),
	REFTYPE( "CASE", cases ),
	REFTYPE( "NEWS", newsarticle ),
	REFTYPE( "MPCT", generic ),
	REFTYPE( "PCOMM", communication ),
	REFTYPE( "PAMP", pamphlet ),
	REFTYPE( "ELEC", electric ),
	REFTYPE( "THES", thesis ),
	REFTYPE( "RPRT", report ),
	REFTYPE( "ABST", abstract ),
	REFTYPE( "COMP", program ),
	REFTYPE( "PAT", patent ),
	REFTYPE( "MAP", map ),
	REFTYPE( "UNPB", unpublished ),
};

int ris_nall = sizeof( ris_all ) / sizeof( variants );

