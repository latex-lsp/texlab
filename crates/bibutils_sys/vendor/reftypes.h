/*
 * reftypes.h
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef REFTYPES_H
#define REFTYPES_H

#define REFTYPE_CHATTY  (0)
#define REFTYPE_SILENT  (1)

/* Reftypes handled by core code */
#define ALWAYS          (0)
#define DEFAULT         (1)
#define SKIP            (2)

/* Reftypes to be handled by converters */
#define SIMPLE          (3)
#define TYPE            (4)
#define PERSON          (5)
#define DATE            (6)
#define PAGES           (7)
#define SERIALNO        (8)
#define TITLE           (9)
#define NOTES           (10)
#define DOI             (11)
#define HOWPUBLISHED    (12)
#define LINKEDFILE      (13)
#define KEYWORD         (14)
#define URL             (15)
#define GENRE           (16)
#define BT_SENTE        (17) /* Bibtex 'Sente' */
#define BT_EPRINT       (18) /* Bibtex 'Eprint' */
#define BT_ORG          (19) /* Bibtex Organization */
#define BLT_THESIS_TYPE (20) /* Biblatex Thesis Type */
#define BLT_SCHOOL      (21) /* Biblatex School */
#define BLT_EDITOR      (22) /* Biblatex Editor */
#define BLT_SUBTYPE     (23) /* Biblatex entrysubtype */
#define BLT_SKIP        (24) /* Biblatex Skip Entry */
#define EPRINT          (25)
#define NUM_REFTYPES    (26)

typedef struct {
	char *oldstr;
	char *newstr;
	int  processingtype;
	int  level;
} lookups;

typedef struct {
	char    type[25];
	lookups *tags;
	int     ntags;
} variants;

int get_reftype( const char *q, long refnum, char *progname, variants *all, int nall, char *tag, int *is_default, int chattiness );
int process_findoldtag( const char *oldtag, int reftype, variants all[], int nall );
int translate_oldtag( const char *oldtag, int reftype, variants all[], int nall, int *processingtype, int *level, char **newtag );

#endif
