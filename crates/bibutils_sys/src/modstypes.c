/*
 * modstypes.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *      
 */
#include <stdio.h>
#include <string.h>
#include "modstypes.h"

/* Conversion information for identifier type attributes:
 *
 *       <identifier type="issn">XXXX-XXXX</identifier>
 *
 */
convert identifier_types[] = {
	{ "citekey",       "REFNUM"    },
	{ "issn",          "ISSN"      },
	{ "isbn",          "ISBN"      },
	{ "doi",           "DOI"       },
	{ "url",           "URL"       },
	{ "uri",           "URL"       },
	{ "pubmed",        "PMID",     },
	{ "medline",       "MEDLINE"   },
	{ "pmc",           "PMC"       },
	{ "pii",           "PII"       },
	{ "isi",           "ISIREFNUM" },
	{ "lccn",          "LCCN"      },
	{ "serial number", "SERIALNUMBER" },
	{ "accessnum",     "ACCESSNUM"    }
};

int nidentifier_types = sizeof( identifier_types ) / sizeof( identifier_types[0] );

char *
mods_find_attrib( char *internal_name, convert *data, int ndata )
{
	int i;
	for ( i=0; i<ndata; ++i ) {
		if ( !strcasecmp( data[i].internal, internal_name ) )
			return data[i].mods;
	}
	return NULL;
}

char *
mods_find_internal( char *mods_name, convert *data, int ndata )
{
	int i;
	for ( i=0; i<ndata; ++i ) {
		if ( !strcasecmp( data[i].mods, mods_name ) )
			return data[i].internal;
	}
	return NULL;
}
