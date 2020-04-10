/*
 * endxmlin.c
 *
 * Copyright (c) Chris Putnam 2006-2019
 *
 * Program and source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include "str.h"
#include "str_conv.h"
#include "fields.h"
#include "name.h"
#include "xml.h"
#include "xml_encoding.h"
#include "reftypes.h"
#include "bibformats.h"

typedef struct {
	char *attrib;
	char *internal;
} attribs;

extern variants end_all[];
extern int end_nall;

static int endxmlin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int endxmlin_processf( fields *endin, const char *p, const char *filename, long nref, param *pm );
extern int endin_typef( fields *endin, const char *filename, int nrefs, param *p );
extern int endin_convertf( fields *endin, fields *info, int reftype, param *p );
extern int endin_cleanf( bibl *bin, param *p );


/*****************************************************
 PUBLIC: void endxmlin_initparams()
*****************************************************/
int
endxmlin_initparams( param *pm, const char *progname )
{
	pm->readformat       = BIBL_ENDNOTEXMLIN;
	pm->charsetin        = BIBL_CHARSET_DEFAULT;
	pm->charsetin_src    = BIBL_SRC_DEFAULT;
	pm->latexin          = 0;
	pm->xmlin            = 1;
	pm->utf8in           = 1;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->output_raw       = 0;

	pm->readf    = endxmlin_readf;
	pm->processf = endxmlin_processf;
	pm->cleanf   = NULL;
	pm->typef    = endin_typef;
	pm->convertf = endin_convertf;
	pm->all      = end_all;
	pm->nall     = end_nall;

	slist_init( &(pm->asis) );
	slist_init( &(pm->corps) );

	if ( !progname ) pm->progname = NULL;
	else {
		pm->progname = strdup( progname );
		if ( !pm->progname ) return BIBL_ERR_MEMERR;
	}

	return BIBL_OK;
}

/*****************************************************
 PUBLIC: int endxmlin_readf()
*****************************************************/

static int
xml_readmore( FILE *fp, char *buf, int bufsize, int *bufpos )
{
	if ( !feof( fp ) && fgets( buf, bufsize, fp ) ) return 0;
	return 1;
}

static int
endxmlin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	int haveref = 0, inref = 0, done = 0, file_charset = CHARSET_UNKNOWN, m;
	char *startptr = NULL, *endptr = NULL;
	str tmp;

	str_init( &tmp );

	while ( !haveref && !done ) {

		if ( !line->data ) {
			done = xml_readmore( fp, buf, bufsize, bufpos );
			str_strcatc( line, buf );
		}

		if ( !inref ) {
			startptr = xml_find_start( line->data, "RECORD" );
			if ( startptr ) inref = 1;
		}
		else    endptr = xml_find_end( line->data, "RECORD" );

		/* If no <record> tag, we can trim up to last 8 bytes */
		/* Emptying string can lose fragments of <record> tag */
		if ( !startptr ) {
			if ( line->len > 8 ) {
				int n = 8;
				char *p = &(line->data[line->len-1]);
				while ( *p && n ) { p--; n--; }
				str_segdel( line, line->data, p );
			}
		}

		/* ...entire reference is not in line, read more */
		if ( !startptr || !endptr ) {
			done = xml_readmore( fp, buf, bufsize, bufpos );
			str_strcatc( line, buf );
		}
		/* ...we can reallocate in str_strcat; must re-find the tags */
		else {
			startptr = xml_find_start( line->data, "RECORD" );
			endptr = xml_find_end( line->data, "RECORD" );
			str_segcpy( reference, startptr, endptr );
			/* clear out information in line */
			str_strcpyc( &tmp, endptr );
			str_strcpy( line, &tmp );
			haveref = 1;
		}

		m = xml_getencoding( line );
		if ( m!=CHARSET_UNKNOWN ) file_charset = m;

	}

	str_free( &tmp );
	*fcharset = file_charset;

	return haveref;
}

/*****************************************************
 PUBLIC: int endxmlin_processf()
*****************************************************/

/*
 * add data to fields
 */

/*
 * handle fields with (potentially) several style pieces
 *
 *   <datatype>
 *          <style>aaaaa</style>
 *   </datatype>
 *
 *   <datatype>aaaaaa</datatype>
 *
 *   <datatype>
 *          <style>aaa</style><style>aaaa</style>
 *   </datatype>
 */
static int
endxmlin_datar( xml *node, str *s )
{
	int status;

	if ( xml_has_value( node ) ) {
		str_strcat( s, &(node->value) );
		if ( str_memerr( s ) ) return BIBL_ERR_MEMERR;
	}
	if ( node->down && xml_tag_matches( node->down, "style" ) ) {
		status = endxmlin_datar( node->down, s );
		if ( status!=BIBL_OK ) return status;
	}
	if ( xml_tag_matches( node, "style" ) && node->next ) {
		status = endxmlin_datar( node->next, s );
		if ( status!=BIBL_OK ) return status;
	}

	return BIBL_OK;
}

static int
endxmlin_data( xml *node, char *inttag, fields *info, int level )
{
	int status;
	str s;

	str_init( &s );

	status = endxmlin_datar( node, &s );
	if ( status!=BIBL_OK ) return status;

	if ( str_has_value( &s ) ) {
		status = fields_add( info, inttag, s.data, level );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	str_free( &s );
	return BIBL_OK;
}

/* <titles>
 *    <title>
 *       <style>ACTUAL TITLE HERE</style><style>MORE TITLE</style>
 *    </title>
 * </titles>
 */
static int
endxmlin_titles( xml *node, fields *info )
{
	attribs a[] = {
		{ "title", "%T" },
		{ "secondary-title", "%B" },
		{ "tertiary-title", "%S" },
		{ "alt-title", "%!" },
		{ "short-title", "SHORTTITLE" },
	};
	int i, status, n = sizeof( a ) / sizeof ( a[0] );
	str title;
	str_init( &title );
	for ( i=0; i<n; ++i ) {
		if ( xml_tag_matches( node, a[i].attrib ) && node->down ) {
			str_empty( &title );
			status = endxmlin_datar( node, &title );
			if ( status!=BIBL_OK ) return BIBL_ERR_MEMERR;
			str_trimstartingws( &title );
			str_trimendingws( &title );
			status = fields_add( info, a[i].internal, title.data, 0);
			if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
	}
	if ( node->next ) {
		status = endxmlin_titles( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	str_free( &title );
	return BIBL_OK;
}

/* <contributors>
 *    <secondary-authors>
 *        <author>
 *             <style>ACTUAL AUTHORS HERE</style>
 *        </author>
 *    </secondary-authors>
 * </contributors>
 */
/* <!ATTLIST author
 *      corp-name CDATA #IMPLIED
 *      first-name CDATA #IMPLIED
 *      initials CDATA #IMPLIED
 *      last-name CDATA #IMPLIED
 *      middle-initial CDATA #IMPLIED
 *      role CDATA #IMPLIED
 *      salutation CDATA #IMPLIED
 *      suffix CDATA #IMPLIED
 *      title CDATA #IMPLIED
 * >
 *
 */
static int
endxmlin_contributor( xml *node, fields *info, char *int_tag, int level )
{
	int status;
	status = endxmlin_data( node, int_tag, info, level );
	if ( status!=BIBL_OK ) return status;
	if ( node->next ) {
		status = endxmlin_contributor( node->next, info, int_tag, level );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
endxmlin_contributors( xml *node, fields *info )
{
	attribs a[] = {
		{ "authors", "%A" },
		{ "secondary-authors", "%E" },
		{ "tertiary-authors", "%Y" },
		{ "subsidiary-authors", "%?" },
		{ "translated-authors", "%?" },
	};
	int i, status, n = sizeof( a ) / sizeof ( a[0] );
	for ( i=0; i<n; ++i ) {
		if ( xml_tag_matches( node, a[i].attrib ) && node->down ) {
			status = endxmlin_contributor( node->down, info, a[i].internal, 0 );
			if ( status!=BIBL_OK ) return status;
		}
	}
	if ( node->next ) {
		status = endxmlin_contributors( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
endxmlin_keyword( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches( node, "keyword" ) ) {
		status = endxmlin_data( node, "%K", info, 0 );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = endxmlin_keyword( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
endxmlin_keywords( xml *node, fields *info )
{
	if ( node->down && xml_tag_matches( node->down, "keyword" ) )
		return endxmlin_keyword( node->down, info );
	return BIBL_OK;
}

/*
 *<electronic-resource-num><style face="normal" font="default" 
 * size="100%">10.1007/BF00356334</style></electronic-resource-num>
 */
static int
endxmlin_ern( xml *node, fields *info )
{
	if ( xml_tag_matches( node, "electronic-resource-num" ) )
		return endxmlin_data( node, "DOI", info, 0 );
	return BIBL_OK;
}

static int
endxmlin_language( xml *node, fields *info )
{
	if ( xml_tag_matches( node, "language" ) )
		return endxmlin_data( node, "%G", info, 0 );
	return BIBL_OK;
}

/*
 * <urls>
 *    <pdf-urls>
 *           <url>internal-pdf://Zukin_1995_The_Cultures_of_Cities-0000551425/Zukin_1995_The_Cultures_of_Cities.pdf</url>
 *    </pdf-urls>
 * </urls>
 */
static int
endxmlin_fileattach( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches( node, "url" ) ) {
		status = endxmlin_data( node, "FILEATTACH", info, 0 );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->down ) {
		status = endxmlin_fileattach( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = endxmlin_fileattach( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
endxmlin_urls( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches( node, "pdf-urls" ) && node->down ) {
		status = endxmlin_fileattach( node->down, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "url" ) ) {
		status = endxmlin_data( node, "%U", info, 0 );
		if ( status!=BIBL_OK ) return status;
	} else {
		if ( node->down ) {
			if ( xml_tag_matches( node->down, "related-urls" ) ||
			     xml_tag_matches( node->down, "pdf-urls" ) ||
			     xml_tag_matches( node->down, "url" ) ) {
				status = endxmlin_urls( node->down, info );
				if ( status!=BIBL_OK ) return status;
			}
		}
	}
	if ( node->next ) {
		status = endxmlin_urls( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
endxmlin_pubdates( xml *node, fields *info )
{
	if ( xml_tag_matches( node, "date" ) )
		return endxmlin_data( node, "%8", info, 0 );
	else {
		if ( node->down && xml_tag_matches( node->down, "date" ) )
			return endxmlin_pubdates( node->down, info );
	}
	return BIBL_OK;
}

static int
endxmlin_dates( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches( node, "year" ) ) {
		status = endxmlin_data( node, "%D", info, 0 );
		if ( status!=BIBL_OK ) return status;
	} else {
		if ( node->down ) {
			if ( xml_tag_matches( node->down, "year" ) ) {
				status = endxmlin_dates( node->down, info );
				if ( status!=BIBL_OK ) return status;
			}
			if ( xml_tag_matches( node->down, "pub-dates" ) ) {
				status = endxmlin_pubdates( node->down, info );
				if ( status!=BIBL_OK ) return status;
			}
		}
	}
	if ( node->next ) {
		status = endxmlin_dates( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/*
 * <ref-type name="Journal Article">17</ref-type>
 */
static int
endxmlin_reftype( xml *node, fields *info )
{
	int status;
	str *s;

	s = xml_attribute( node, "name" );
	if ( str_has_value( s ) ) {
		status = fields_add( info, "%0", str_cstr( s ), 0 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	return BIBL_OK;
}

static int
endxmlin_record( xml *node, fields *info )
{
	attribs a[] = {
		{ "volume", "%V" },
		{ "num-vol", "%6" },
		{ "pages",  "%P" },
		{ "number", "%N" },
		{ "issue",  "%N" },
		{ "label",  "%F" },
		{ "auth-address", "%C" },
		{ "auth-affiliation", "%C" },
		{ "pub-location", "%C" },
		{ "publisher", "%I" },
		{ "abstract", "%X" },
		{ "edition", "%7" },
		{ "reprint-edition", "%)" },
		{ "section", "%&" },
		{ "accession-num", "%M" },
		{ "call-num", "%L" },
		{ "isbn", "%@" },
		{ "notes", "%O" },
		{ "custom1", "%1" },
		{ "custom2", "%2" },
		{ "custom3", "%3" },
		{ "custom4", "%4" },
		{ "custom5", "%#" },
		{ "custom6", "%$" },
	};
	int i, status, n = sizeof ( a ) / sizeof( a[0] );
	if ( xml_tag_matches( node, "DATABASE" ) ) {
	} else if ( xml_tag_matches( node, "SOURCE-APP" ) ) {
	} else if ( xml_tag_matches( node, "REC-NUMBER" ) ) {
	} else if ( xml_tag_matches( node, "ref-type" ) ) {
		status = endxmlin_reftype( node, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "contributors" ) ) {
		if ( node->down ) {
			status = endxmlin_contributors( node->down, info );
			if ( status!=BIBL_OK ) return status;
		}
	} else if ( xml_tag_matches( node, "titles" ) ) {
		if ( node->down ) endxmlin_titles( node->down, info );
	} else if ( xml_tag_matches( node, "keywords" ) ) {
		status = endxmlin_keywords( node, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "urls" ) ) {
		status = endxmlin_urls( node, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "electronic-resource-num" ) ) {
		status = endxmlin_ern( node, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "dates" ) ) {
		status = endxmlin_dates( node, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "language" ) ) {
		status = endxmlin_language( node, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( xml_tag_matches( node, "periodical" ) ) {
	} else if ( xml_tag_matches( node, "secondary-volume" ) ) {
	} else if ( xml_tag_matches( node, "secondary-issue" ) ) {
	} else if ( xml_tag_matches( node, "reprint-status" ) ) {
	} else if ( xml_tag_matches( node, "orig-pub" ) ) {
	} else if ( xml_tag_matches( node, "report-id" ) ) {
	} else if ( xml_tag_matches( node, "coden" ) ) {
	} else if ( xml_tag_matches( node, "caption" ) ) {
	} else if ( xml_tag_matches( node, "research-notes" ) ) {
	} else if ( xml_tag_matches( node, "work-type" ) ) {
	} else if ( xml_tag_matches( node, "reviewed-item" ) ) {
	} else if ( xml_tag_matches( node, "availability" ) ) {
	} else if ( xml_tag_matches( node, "remote-source" ) ) {
	} else if ( xml_tag_matches( node, "meeting-place" ) ) {
	} else if ( xml_tag_matches( node, "work-location" ) ) {
	} else if ( xml_tag_matches( node, "work-extent" ) ) {
	} else if ( xml_tag_matches( node, "pack-method" ) ) {
	} else if ( xml_tag_matches( node, "size" ) ) {
	} else if ( xml_tag_matches( node, "repro-ratio" ) ) {
	} else if ( xml_tag_matches( node, "remote-database-name" ) ) {
	} else if ( xml_tag_matches( node, "remote-database-provider" ) ) {
	} else if ( xml_tag_matches( node, "access-date" ) ) {
	} else if ( xml_tag_matches( node, "modified-data" ) ) {
	} else if ( xml_tag_matches( node, "misc1" ) ) {
	} else if ( xml_tag_matches( node, "misc2" ) ) {
	} else if ( xml_tag_matches( node, "misc3" ) ) {
	} else {
		for ( i=0; i<n; ++i ) {
			if ( xml_tag_matches( node, a[i].attrib ) ) {
				status = endxmlin_data( node, a[i].internal, info, 0 );
				if ( status!=BIBL_OK ) return status;
			}
		}
	}
	if ( node->next ) {
		status = endxmlin_record( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
endxmlin_assembleref( xml *node, fields *info )
{
	int status;
	if ( str_is_empty( &(node->tag) ) ) {
		if ( node->down )
			return endxmlin_assembleref( node->down, info );
	} else if ( xml_tag_matches( node, "RECORD" ) ) {
		if ( node->down ) {
			status = endxmlin_record( node->down, info );
			if ( status!=BIBL_OK ) return status;
		}
	}
	return BIBL_OK;
}

/* endxmlin_processf first operates by converting to endnote input
 * the endnote->mods conversion happens in convertf.
 *
 * this is necessary as the xml format is as nasty and as overloaded
 * as the tags used in the Refer format output
 */
static int
endxmlin_processf( fields *fin, const char *data, const char *filename, long nref, param *pm )
{
	int status;
	xml top;

	xml_init( &top );
	xml_parse( data, &top );
	status = endxmlin_assembleref( &top, fin );
	xml_free( &top );

	if ( status==BIBL_OK ) return 1;
	return 0;
}
