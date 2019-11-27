/*
 * wordin.c
 *
 * Copyright (c) Chris Putnam 2010-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include "is_ws.h"
#include "str.h"
#include "str_conv.h"
#include "fields.h"
#include "xml.h"
#include "xml_encoding.h"
#include "bibformats.h"

static int wordin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int wordin_processf( fields *wordin, const char *data, const char *filename, long nref, param *p );


/*****************************************************
 PUBLIC: void wordin_initparams()
*****************************************************/

int
wordin_initparams( param *pm, const char *progname )
{
	pm->readformat       = BIBL_WORDIN;
	pm->charsetin        = BIBL_CHARSET_DEFAULT;
	pm->charsetin_src    = BIBL_SRC_DEFAULT;
	pm->latexin          = 0;
	pm->xmlin            = 1;
	pm->utf8in           = 1;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->output_raw       = BIBL_RAW_WITHMAKEREFID |
	                      BIBL_RAW_WITHCHARCONVERT;

	pm->readf    = wordin_readf;
	pm->processf = wordin_processf;
	pm->cleanf   = NULL;
	pm->typef    = NULL;
	pm->convertf = NULL;
	pm->all      = NULL;
	pm->nall     = 0;

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
 PUBLIC: int wordin_readf()
*****************************************************/

static char *
wordin_findstartwrapper( char *buf, int *ntype )
{
	return xml_find_start( buf, "b:Source" );
}

static char *
wordin_findendwrapper( char *buf, int ntype )
{
	return xml_find_end( buf, "b:Source" );
}

static int
wordin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	str tmp;
	char *startptr = NULL, *endptr;
	int haveref = 0, inref = 0, file_charset = CHARSET_UNKNOWN, m, type = 1;
	str_init( &tmp );
	while ( !haveref && str_fget( fp, buf, bufsize, bufpos, line ) ) {
		if ( str_cstr( line ) ) {
			m = xml_getencoding( line );
			if ( m!=CHARSET_UNKNOWN ) file_charset = m;
		}
		if ( str_cstr( line ) ) {
			startptr = wordin_findstartwrapper( str_cstr( line ), &type );
		}
		if ( startptr || inref ) {
			if ( inref ) str_strcat( &tmp, line );
			else {
				str_strcatc( &tmp, startptr );
				inref = 1;
			}
			endptr = wordin_findendwrapper( str_cstr( &tmp ), type );
			if ( endptr ) {
				str_segcpy( reference, str_cstr( &tmp ), endptr );
				haveref = 1;
			}
		}
	}
	str_free( &tmp );
	*fcharset = file_charset;
	return haveref;
}

/*****************************************************
 PUBLIC: int wordin_processf()
*****************************************************/

typedef struct xml_convert {
	char *in;       /* The input tag */
	char *a, *aval; /* The attribute="attribute_value" pair, if nec. */
	char *out;      /* The output tag */
	int level;
} xml_convert;

/* wordin_person_last()
 *
 * From an xml list, extract the value from the first entry
 * of <b:Last>xxxx</b:Last> and copy into name
 *
 * Additional <b:Last>yyyyy</b:Last> will be ignored.
 *
 * Returns BIBL_ERR_MEMERR on memory error, BIBL_OK otherwise.
 */
static int
wordin_person_last( xml *node, str *name )
{
	while ( node && !xml_tag_matches( node, "b:Last" ) )
		node = node->next;
	if ( xml_has_value( node ) ) {
		str_strcpy( name, xml_value( node ) );
		if ( str_memerr( name ) ) return BIBL_ERR_MEMERR;
	}
	return BIBL_OK;
}

/* wordin_person_first()
 *
 * From an xml list, extract the value of any
 * <b:First>xxxx</b:First> and append "|xxxx" to name.
 *
 * Returns BIBL_ERR_MEMERR on memory error, BIBL_OK otherwise
 */
static int
wordin_person_first( xml *node, str *name )
{
	for ( ; node; node=node->next ) {
		if ( !xml_tag_matches( node, "b:First" ) ) continue;
		if ( xml_has_value( node ) ) {
			if ( str_has_value( name ) ) str_addchar( name, '|' );
			str_strcat( name, xml_value( node ) );
			if ( str_memerr( name ) ) return BIBL_ERR_MEMERR;
		}
	}
	return BIBL_OK;
}

static int
wordin_person( xml *node, fields *info, char *type )
{
	int status, ret = BIBL_OK;
	str name;

	str_init( &name );

	status = wordin_person_last( node, &name );
	if ( status!=BIBL_OK ) {
		ret = status;
		goto out;
	}

	status = wordin_person_first( node, &name );
	if ( status!=BIBL_OK ) {
		ret = status;
		goto out;
	}

	status = fields_add( info, type, str_cstr( &name ), 0 );
	if ( status != FIELDS_OK ) ret = BIBL_ERR_MEMERR;
out:
	str_free( &name );
	return ret;
}

static int
wordin_people( xml *node, fields *info, char *type )
{
	int ret = BIBL_OK;
	if ( xml_tag_matches( node, "b:Author" ) && node->down ) {
		ret = wordin_people( node->down, info, type );
	} else if ( xml_tag_matches( node, "b:NameList" ) && node->down ) {
		ret = wordin_people( node->down, info, type );
	} else if ( xml_tag_matches( node, "b:Person" ) ) {
		if ( node->down ) ret = wordin_person( node->down, info, type );
		if ( ret!=BIBL_OK ) return ret;
		if ( node->next ) ret = wordin_people( node->next, info, type );
	}
	return ret;
}

static int
wordin_pages( xml *node, fields *info )
{
	int i, status, ret = BIBL_OK;
	str sp, ep;
	char *p;

	strs_init( &sp, &ep, NULL );

	p = xml_value_cstr( node );
	while ( *p && *p!='-' )
		str_addchar( &sp, *p++ );
	if ( str_memerr( &sp ) ) {
		ret = BIBL_ERR_MEMERR;
		goto out;
	}

	if ( *p=='-' ) p++;
	while ( *p )
		str_addchar( &ep, *p++ );
	if ( str_memerr( &ep ) ) {
		ret = BIBL_ERR_MEMERR;
		goto out;
	}

	if ( str_has_value( &sp ) ) {
		status = fields_add( info, "PAGES:START", str_cstr( &sp ), 1 );
		if ( status!=FIELDS_OK ) {
			ret = BIBL_ERR_MEMERR;
			goto out;
		}
	}

	if ( str_has_value( &ep ) ) {
		if ( sp.len > ep.len ) {
			for ( i=sp.len-ep.len; i<sp.len; ++i )
				sp.data[i] = ep.data[i-sp.len+ep.len];
			status = fields_add( info, "PAGES:STOP", str_cstr( &sp ), 1 );
		} else
			status = fields_add( info, "PAGES:STOP", str_cstr( &ep ), 1 );
		if ( status!=FIELDS_OK ) {
			ret = BIBL_ERR_MEMERR;
			goto out;
		}
	}

out:
	strs_free( &sp, &ep, NULL );
	return ret;
}

static int
wordin_reference( xml *node, fields *info )
{
	int status, ret = BIBL_OK;
	if ( xml_has_value( node ) ) {
		if ( xml_tag_matches( node, "b:Tag" ) ) {
			status = fields_add( info, "REFNUM", xml_value_cstr( node ), 0 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:SourceType" ) ) {
		} else if ( xml_tag_matches( node, "b:City" ) ) {
			status = fields_add( info, "ADDRESS", xml_value_cstr( node ), 0 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:Publisher" ) ) {
			status = fields_add( info, "PUBLISHER", xml_value_cstr( node ), 0 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:Title" ) ) {
			status = fields_add( info, "TITLE", xml_value_cstr( node ), 0 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:JournalName" ) ) {
			status = fields_add( info, "TITLE", xml_value_cstr( node ), 1 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:Volume" ) ) {
			status = fields_add( info, "VOLUME", xml_value_cstr( node ), 1 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:Comments" ) ) {
			status = fields_add( info, "NOTES", xml_value_cstr( node ), 0 );
			if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
		} else if ( xml_tag_matches( node, "b:Pages" ) ) {
			ret = wordin_pages( node, info );
		} else if ( xml_tag_matches( node, "b:Author" ) && node->down ) {
			ret = wordin_people( node->down, info, "AUTHOR" );
		} else if ( xml_tag_matches( node, "b:Editor" ) && node->down ) {
			ret = wordin_people( node->down, info, "EDITOR" );
		}
	}
	if ( ret==BIBL_OK && node->next ) wordin_reference( node->next, info );
	return ret;
}

static int
wordin_assembleref( xml *node, fields *info )
{
	int ret = BIBL_OK;
	if ( xml_tag_matches( node, "b:Source" ) ) {
		if ( node->down ) ret = wordin_reference( node->down, info );
	} else if ( str_is_empty( &(node->tag) ) && node->down ) {
		ret = wordin_assembleref( node->down, info );
	}
	return ret;
}

static int
wordin_processf( fields *wordin, const char *data, const char *filename, long nref, param *p )
{
	int status, ret = 1;
	xml top;

	xml_init( &top );
	xml_parse( data, &top );
	status = wordin_assembleref( &top, wordin );
	xml_free( &top );

	if ( status==BIBL_ERR_MEMERR ) ret = 0;
	return ret;
}
