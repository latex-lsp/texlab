/*
 * nbibin.c
 *
 * Copyright (c) Chris Putnam 2016-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "is_ws.h"
#include "str.h"
#include "str_conv.h"
#include "fields.h"
#include "name.h"
#include "title.h"
#include "url.h"
#include "serialno.h"
#include "reftypes.h"
#include "bibformats.h"
#include "generic.h"

extern variants nbib_all[];
extern int nbib_nall;

/*****************************************************
 PUBLIC: void nbib_initparams()
*****************************************************/

static int nbib_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int nbib_processf( fields *nbib, const char *p, const char *filename, long nref, param *pm );
static int nbib_typef( fields *nbib, const char *filename, int nref, param *p );
static int nbib_convertf( fields *nbib, fields *info, int reftype, param *p );

int
nbibin_initparams( param *pm, const char *progname )
{
	pm->readformat       = BIBL_NBIBIN;
	pm->charsetin        = BIBL_CHARSET_DEFAULT;
	pm->charsetin_src    = BIBL_SRC_DEFAULT;
	pm->latexin          = 0;
	pm->xmlin            = 0;
	pm->utf8in           = 0;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->output_raw       = 0;

	pm->readf    = nbib_readf;
	pm->processf = nbib_processf;
	pm->cleanf   = NULL;
	pm->typef    = nbib_typef;
	pm->convertf = nbib_convertf;
	pm->all      = nbib_all;
	pm->nall     = nbib_nall;

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
 PUBLIC: int nbib_readf()
*****************************************************/

/* RIS definition of a tag is strict:
    character 1 = uppercase alphabetic character
    character 2 = uppercase alphabetic character
    character 3 = character or space (ansi 32)
    character 4 = character or space (ansi 32)
    character 5 = dash (ansi 45)
    character 6 = space (ansi 32)
*/
static inline int
is_upperchar( const char c )
{
	if ( c>='A' && c<='Z' ) return 1;
	else return 0;
}

static inline int
is_upperchar_space( const char c )
{
	if ( c==' ' ) return 1;
	if ( c>='A' && c<='Z' ) return 1;
	else return 0;
}

static int
nbib_istag( const char *buf )
{
	if ( !is_upperchar( buf[0] ) ) return 0;
	if ( !is_upperchar( buf[1] ) ) return 0;
	if ( !is_upperchar_space( buf[2] ) ) return 0;
	if ( !is_upperchar_space( buf[3] ) ) return 0;
	if (buf[4]!='-') return 0;
	if (buf[5]!=' ') return 0;
	return 1;
}

static int
readmore( FILE *fp, char *buf, int bufsize, int *bufpos, str *line )
{
	if ( line->len ) return 1;
	else return str_fget( fp, buf, bufsize, bufpos, line );
}

static int
skip_utf8_bom( str *line, int *fcharset )
{
	unsigned char *up;

	if ( line->len < 3 ) return 0;

	up = ( unsigned char *) str_cstr( line );
	if ( up[0]==0xEF && up[1]==0xBB && up[2]==0xBF ) {
		*fcharset = CHARSET_UNICODE;
		return 3;
	}

	return 0;
}

static int
nbib_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	int n, haveref = 0, inref = 0, readtoofar = 0;
	char *p;

	*fcharset = CHARSET_UNKNOWN;

	while ( !haveref && readmore( fp, buf, bufsize, bufpos, line ) ) {

		/* ...references are terminated by an empty line */
		if ( str_is_empty( line ) ) {
			if ( reference->len ) haveref = 1;
			continue;
		}

		/* ...recognize and skip over UTF8 BOM */
		n = skip_utf8_bom( line, fcharset );
		p = &( line->data[n] );

		/* Each reference starts with 'PMID- ' && ends with blank line */
		if ( strncmp(p,"PMID- ",6)==0 ) {
			if ( !inref ) {
				inref = 1;
			} else {
				/* we've read too far.... */
				readtoofar = 1;
				inref = 0;
			}
		}
		if ( nbib_istag( p ) ) {
			if ( !inref ) {
				fprintf(stderr,"Warning.  Tagged line not "
					"in properly started reference.\n");
				fprintf(stderr,"Ignored: '%s'\n", p );
			} else if ( !strncmp(p,"ER  -",5) ) {
				inref = 0;
			} else {
				str_addchar( reference, '\n' );
				str_strcatc( reference, p );
			}
		}
		/* not a tag, but we'll append to last values ...*/
		else if ( inref && strlen( p ) >= 6 ) {
			str_strcatc( reference, p+5 );
		}
		if ( !readtoofar ) str_empty( line );
	}
	if ( inref ) haveref = 1;
	return haveref;
}

/*****************************************************
 PUBLIC: int nbib_processf()
*****************************************************/

static const char*
process_line2( str *tag, str *value, const char *p )
{
	while ( *p==' ' || *p=='\t' ) p++;
	while ( *p && *p!='\r' && *p!='\n' )
		str_addchar( value, *p++ );
	while ( *p=='\r' || *p=='\n' ) p++;
	return p;
}

static const char*
process_line( str *tag, str *value, const char *p )
{
	int i;

	i = 0;
	while ( i<6 && *p ) {
		if ( *p!=' ' && *p!='-' ) str_addchar( tag, *p );
		p++;
		i++;
	}
	while ( *p==' ' || *p=='\t' ) p++;
	while ( *p && *p!='\r' && *p!='\n' )
		str_addchar( value, *p++ );
	str_trimendingws( value );
	while ( *p=='\n' || *p=='\r' ) p++;
	return p;
}

static int
nbib_processf( fields *nbib, const char *p, const char *filename, long nref, param *pm )
{
	str tag, value;
	int status, n;

	strs_init( &tag, &value, NULL );

	while ( *p ) {
		if ( nbib_istag( p ) )
			p = process_line( &tag, &value, p );
		/* no anonymous fields allowed */
		if ( str_has_value( &tag ) ) {
			status = fields_add( nbib, str_cstr( &tag ), str_cstr( &value ), 0 );
			if ( status!=FIELDS_OK ) return 0;
		} else {
			p = process_line2( &tag, &value, p );
			n = fields_num( nbib );
			if ( value.len && n>0 ) {
				str *od;
				od = fields_value( nbib, n-1, FIELDS_STRP );
				str_addchar( od, ' ' );
				str_strcat( od, &value );
			}
		}
		strs_empty( &tag, &value, NULL );
	}

	strs_free( &tag, &value, NULL );
	return 1;
}

/*****************************************************
 PUBLIC: int nbib_typef()
*****************************************************/

/*
 * PT  - Case Reports
 * PT  - Journal Article
 * PT  - Research Support, N.I.H., Extramural
 * PT  - Review
 */
static int
nbib_typef( fields *nbib, const char *filename, int nref, param *p )
{
	int reftype = 0, nrefname, is_default;
	char *typename, *refname = "";
	vplist_index i;
	vplist a;

	nrefname  = fields_find( nbib, "PMID", LEVEL_MAIN );
	if ( nrefname!=FIELDS_NOTFOUND ) refname = fields_value( nbib, nrefname, FIELDS_CHRP_NOUSE );

	vplist_init( &a );

	fields_findv_each( nbib, LEVEL_MAIN, FIELDS_CHRP_NOUSE, &a, "PT" );
	is_default = 1;
	for ( i=0; i<a.n; ++i ) {
		typename = vplist_get( &a, i );
		reftype  = get_reftype( typename, nref, p->progname, p->all, p->nall, refname, &is_default, REFTYPE_SILENT );
		if ( !is_default ) break;
	}

	if ( a.n==0 )
		reftype = get_reftype( "", nref, p->progname, p->all, p->nall, refname, &is_default, REFTYPE_CHATTY );
	else if ( is_default ) {
                if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
                fprintf( stderr, "Did not recognize type of refnum %d (%s).\n"
                        "\tDefaulting to %s.\n", nref, refname, p->all[0].type );
	}

	vplist_free( &a );

	return reftype;
}

/*****************************************************
 PUBLIC: int nbib_convertf()
*****************************************************/

/* PB  - 2016 May 7 */

static int
nbibin_date( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	int fstatus, status = BIBL_OK;
	str s;
	char *p;

	p = str_cstr( invalue );
	if ( !p ) return status;

	str_init( &s );

	/* ...handle year */
		while ( *p && !is_ws( *p ) ) {
			str_addchar( &s, *p );
			p++;
		}

		if ( str_memerr( &s ) ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}

		if ( str_has_value( &s ) ) {
			fstatus = fields_add( bibout, "DATE:YEAR", str_cstr( &s ), LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) {
				status = BIBL_ERR_MEMERR;
				goto out;
			}
		}

	/* ...handle month */
		str_empty( &s );
		while ( is_ws( *p ) ) p++;
		while ( *p && !is_ws( *p ) ) {
			str_addchar( &s, *p );
			p++;
		}

		if ( str_memerr( &s ) ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}

		if ( str_has_value( &s ) ) {
			fstatus = fields_add( bibout, "DATE:MONTH", str_cstr( &s ), LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) {
				status = BIBL_ERR_MEMERR;
				goto out;
			}
		}

	/* ...handle day */
		str_empty( &s );
		while ( is_ws( *p ) ) p++;
		while ( *p && !is_ws( *p ) ) {
			str_addchar( &s, *p );
			p++;
		}

		if ( str_memerr( &s ) ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}

		if ( str_has_value( &s ) ) {
			fstatus = fields_add( bibout, "DATE:DAY", str_cstr( &s ), LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) {
				status = BIBL_ERR_MEMERR;
				goto out;
			}
		}

out:
	str_free( &s );

	return status;
}

/* the LID and AID fields that can be doi's or pii's */
static int
nbibin_doi( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	int fstatus, sstatus, status = BIBL_OK;
	char *id, *type, *usetag="";
	slist tokens;

	slist_init( &tokens );

	sstatus = slist_tokenize( &tokens, invalue, " ", 1 );
	if ( sstatus!=SLIST_OK ) {
		status = BIBL_ERR_MEMERR;
		goto out;
	}

	if ( tokens.n == 2 ) {
		id   = slist_cstr( &tokens, 0 );
		type = slist_cstr( &tokens, 1 );
		if ( !strcmp( type, "[doi]" ) ) usetag = "DOI";
		else if ( !strcmp( type, "[pii]" ) ) usetag = "PII";
		if ( strlen( outtag ) > 0 ) {
			fstatus = fields_add( bibout, usetag, id, LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) {
				status = BIBL_ERR_MEMERR;
				goto out;
			}
		}
	}
out:
	slist_free( &tokens );
	return status;
}

static int
nbibin_pages( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	int fstatus, status = BIBL_OK;
	str sp, tmp, ep;
	char *p;
	int i;

	p = str_cstr( invalue );
	if ( !p ) return BIBL_OK;

	strs_init( &sp, &tmp, &ep, NULL );

	while ( *p && *p!='-' ) {
		str_addchar( &sp, *p );
		p++;
	}

	if ( str_memerr( &sp ) ) {
		status = BIBL_ERR_MEMERR;
		goto out;
	}

	while ( *p=='-' ) p++;

	while ( *p ) {
		str_addchar( &tmp, *p );
		p++;
	}

	if ( sp.len ) {
		fstatus = fields_add( bibout, "PAGES:START", str_cstr( &sp ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}
	}

	if ( tmp.len ) {
		for ( i=0; i<sp.len - tmp.len; ++i )
			str_addchar( &ep, sp.data[i] );
		str_strcat( &ep, &tmp );

		fstatus = fields_add( bibout, "PAGES:STOP", str_cstr( &ep ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}
	}
out:
	strs_free( &sp, &tmp, &ep, NULL );
	return status;
}

static void
nbib_report_notag( param *p, char *tag )
{
	if ( p->verbose && strcmp( tag, "TY" ) ) {
		if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
		fprintf( stderr, "Did not identify NBIB tag '%s'\n", tag );
	}
}

static int
nbib_convertf( fields *bibin, fields *bibout, int reftype, param *p )
{
	static int (*convertfns[NUM_REFTYPES])(fields *, int i, str *, str *, int, param *, char *, fields *) = {
		// Patch: Fix compilation on MSVC target
		[ 0            ] = generic_null,
		[ 1            ] = generic_null,
		[ 2            ] = generic_null,
		[ 3            ] = generic_null,
		[ 4            ] = generic_null,
		[ 5            ] = generic_null,
		[ 6            ] = generic_null,
		[ 7            ] = generic_null,
		[ 8            ] = generic_null,
		[ 9            ] = generic_null,
		[ 10           ] = generic_null,
		[ 11           ] = generic_null,
		[ 12           ] = generic_null,
		[ 13           ] = generic_null,
		[ 14           ] = generic_null,
		[ 15           ] = generic_null,
		[ 16           ] = generic_null,
		[ 17           ] = generic_null,
		[ 18           ] = generic_null,
		[ 19           ] = generic_null,
		[ 20           ] = generic_null,
		[ 21           ] = generic_null,
		[ 22           ] = generic_null,
		[ 23           ] = generic_null,
		[ 24           ] = generic_null,
		[ 25           ] = generic_null,
		[ SIMPLE       ] = generic_simple,
		[ TITLE        ] = generic_title,
		[ PERSON       ] = generic_person,
		[ SKIP         ] = generic_skip,
		[ DATE         ] = nbibin_date,
		[ PAGES        ] = nbibin_pages,
		[ DOI          ] = nbibin_doi,
        };
	int process, level, i, nfields, status = BIBL_OK;
	str *intag, *invalue;
	char *outtag;

	nfields = fields_num( bibin );

	for ( i=0; i<nfields; ++i ) {
		intag = fields_tag( bibin, i, FIELDS_STRP );
		if ( !translate_oldtag( str_cstr( intag ), reftype, p->all, p->nall, &process, &level, &outtag ) ) {
			nbib_report_notag( p, str_cstr( intag ) );
			continue;
		}
		invalue = fields_value( bibin, i, FIELDS_STRP );

		status = convertfns[ process ] ( bibin, i, intag, invalue, level, p, outtag, bibout );
		if ( status!=BIBL_OK ) return status;
	}

	if ( status==BIBL_OK && p->verbose ) fields_report( bibout, stderr );

	return status;
}
