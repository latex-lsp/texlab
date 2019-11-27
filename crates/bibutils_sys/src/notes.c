/*
 * notes.c
 *
 * Copyright (c) Chris Putnam 2016-2019
 *
 * Program and source code released under the GPL version 2
 *
 */
#include <string.h>
#include "url.h"
#include "notes.h"

/*
 * notes are mostly directly copies; however, lots of formats hide
 * URLs/DOIs in the notes fields. For example:
 *
 * For RIS, Oxford Journals hides DOI in the N1 field.
 * For Endnote, Wiley hides DOI in the %1 field.
 * etc.
 */

typedef struct url_t {
	char *prefix;
	char *tag;
	int offset;
} url_t;

static void
notes_added_url( fields *bibout, str *invalue, int level, int *ok )
{
	url_t prefixes[] = {
		{ "arXiv:",                                    "ARXIV",     6 },
		{ "http://arxiv.org/abs/",                     "ARXIV",    21 },
		{ "jstor:",                                    "JSTOR",     6 },
		{ "http://www.jstor.org/stable/",              "JSTOR",    28 },
		{ "medline:",                                  "MEDLINE",   8 },
		{ "pubmed:",                                   "PMID",      7 },
		{ "http://www.ncbi.nlm.nih.gov/pubmed/",       "PMID",     35 },
		{ "http://www.ncbi.nlm.nih.gov/pmc/articles/", "PMC",      41 },
		{ "http://dx.doi.org/",                        "DOI",      19 },
		{ "isi:",                                      "ISIREFNUM", 4 },
	};
	int nprefixes = sizeof( prefixes ) / sizeof( prefixes[0] );

	const char *p = str_cstr( invalue );
	char *tag = "URL";
	int fstatus;
	int i;

	/* bibtex/biblatex-specific */
	if ( !strncasecmp( p, "\\urllink", 8 ) ) p += 8;
	if ( !strncasecmp( p, "\\url", 4 ) ) p += 4;

	for ( i=0; i<nprefixes; ++i ) {
		if ( !strncasecmp( p, prefixes[i].prefix, prefixes[i].offset ) ) {
			tag = prefixes[i].tag;
			p   = p + prefixes[i].offset;
			break;
		}
	}

	fstatus = fields_add( bibout, tag, p, level );

	if ( fstatus==FIELDS_OK ) *ok = 1;
	else *ok = 0;
}

static int
notes_added_doi( fields *bibout, str *invalue, int level, int *ok )
{
	int doi, fstatus;

	doi = is_doi( str_cstr( invalue ) );

	if ( doi != -1 ) {
		fstatus = fields_add( bibout, "DOI", &(invalue->data[doi]), level );
		if ( fstatus != FIELDS_OK ) *ok = 0;
		return 1;
	}

	else return 0;
}

int
notes_add( fields *bibout, str *invalue, int level )
{
	int fstatus, done = 0, ok = 1;

	if ( !is_embedded_link( str_cstr( invalue ) ) ) {
		fstatus = fields_add( bibout, "NOTES", str_cstr( invalue ), level );
		if ( fstatus != FIELDS_OK ) ok = 0;
	}

	else {

		done = notes_added_doi( bibout, invalue, level, &ok );
		if ( !done ) notes_added_url( bibout, invalue, level, &ok );

	}

	return ok;
}
