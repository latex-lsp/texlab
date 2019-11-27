/*
 * risin.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "str.h"
#include "str_conv.h"
#include "fields.h"
#include "name.h"
#include "title.h"
#include "url.h"
#include "utf8.h"
#include "serialno.h"
#include "reftypes.h"
#include "bibformats.h"
#include "generic.h"
#include "msvc_fix.h"

extern variants ris_all[];
extern int ris_nall;

/*****************************************************
 PUBLIC: void risin_initparams()
*****************************************************/

static int risin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int risin_processf( fields *risin, const char *p, const char *filename, long nref, param *pm );
static int risin_typef( fields *risin, const char *filename, int nref, param *p );
static int risin_convertf( fields *risin, fields *info, int reftype, param *p );

int
risin_initparams( param *pm, const char *progname )
{
	pm->readformat       = BIBL_RISIN;
	pm->charsetin        = BIBL_CHARSET_DEFAULT;
	pm->charsetin_src    = BIBL_SRC_DEFAULT;
	pm->latexin          = 0;
	pm->xmlin            = 0;
	pm->utf8in           = 0;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->output_raw       = 0;

	pm->readf    = risin_readf;
	pm->processf = risin_processf;
	pm->cleanf   = NULL;
	pm->typef    = risin_typef;
	pm->convertf = risin_convertf;
	pm->all      = ris_all;
	pm->nall     = ris_nall;

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
 PUBLIC: int risin_readf()
*****************************************************/

/* RIS definition of a tag is strict:
    character 1 = uppercase alphabetic character
    character 2 = uppercase alphabetic character or digit
    character 3 = space (ansi 32)
    character 4 = space (ansi 32)
    character 5 = dash (ansi 45)
    character 6 = space (ansi 32)

  some sources don't have a space at character 6 if there
  is no data (such as "ER  -" records). Handle this.

  www.omicsonline.org mangles the RIS specification and
  puts _three_ spaces before the dash.  Handle this too.
*/

static int
is_ris_tag( const char *buf )
{
	if ( !isupper( (unsigned char )buf[0] ) ) return 0;
	if ( !( isupper( (unsigned char )buf[1] ) || isdigit( (unsigned char )buf[1] ) ) ) return 0;
	if ( buf[2]!=' ' ) return 0;
	if ( buf[3]!=' ' ) return 0;

	/*...RIS tags with two spaces */
	if ( buf[4]=='-' ) {
		if ( buf[5]==' ' || buf[5]=='\0' || buf[5]=='\n' || buf[5]=='\r' ) return 1;
	}

	/* ...RIS tags with three spaces */
	else if ( buf[4]==' ' ) {
		if ( buf[5]!='-' ) return 0;
		if ( buf[6]==' ' || buf[6]=='\0' || buf[6]=='\n' || buf[6]=='\r' ) return 1;
	}

	return 0;
}

static int
is_ris_start_tag( char *p )
{
	/* ...TY tag that fits specifications */
	if ( !strncmp( p, "TY  - ",  6 ) ) return 1;
	/* ...TY tag with an extra space? */
	if ( !strncmp( p, "TY   - ", 7 ) ) return 1;
	return 0;
}

static int
is_ris_end_tag( char *p )
{
	/* ...ER tag that fits specifications */
	if ( !strncmp( p, "ER  -",  5 ) ) return 1;
	/* ...ER tag with an extra space? */
	if ( !strncmp( p, "ER   -", 6 ) ) return 1;
	return 0;
}

static int
readmore( FILE *fp, char *buf, int bufsize, int *bufpos, str *line )
{
	if ( line->len ) return 1;
	else return str_fget( fp, buf, bufsize, bufpos, line );
}

static int
risin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	int haveref = 0, inref = 0, readtoofar = 0;
	char *p;

	*fcharset = CHARSET_UNKNOWN;

	while ( !haveref && readmore( fp, buf, bufsize, bufpos, line ) ) {

		if ( str_is_empty( line ) ) continue;

		p = str_cstr( line );

		if ( utf8_is_bom( p ) ) {
			*fcharset = CHARSET_UNICODE;
			p += 3;
		}

		/* References are bounded by tags 'TY  - ' && 'ER  - ' */
		if ( is_ris_start_tag( p ) ) {
			if ( !inref ) inref = 1;
			else {
				/* we've read too far.... */
				readtoofar = 1;
				inref = 0;
			}
		}

		if ( is_ris_tag( p ) ) {
			if ( !inref ) {
				fprintf(stderr,"Warning.  Tagged line not "
					"in properly started reference.\n");
				fprintf(stderr,"Ignored: '%s'\n", p );
			} else if ( is_ris_end_tag( p ) ) {
				inref = 0;
			} else {
				str_addchar( reference, '\n' );
				str_strcatc( reference, p );
			}
		}
		/* not a tag, but we'll append to last values ...*/
		else if ( inref && !is_ris_end_tag( p ) ) {
			str_addchar( reference, '\n' );
			str_strcatc( reference, p );
		}
		if ( !inref && reference->len ) haveref = 1;
		if ( !readtoofar ) str_empty( line );
	}

	if ( inref ) haveref = 1;

	return haveref;
}

/*****************************************************
 PUBLIC: int risin_processf()
*****************************************************/

static const char*
process_untagged_line( str *value, const char *p )
{
	while ( *p==' ' || *p=='\t' ) p++;
	while ( *p && *p!='\r' && *p!='\n' )
		str_addchar( value, *p++ );
	while ( *p=='\r' || *p=='\n' ) p++;
	return p;
}

static const char*
process_tagged_line( str *tag, str *value, const char *p )
{
	int i = 0;

	while ( i<6 && *p && *p!='\n' && *p!='\r' ) {
		if ( i<2 ) str_addchar( tag, *p );
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
merge_tag_value( fields *risin, str *tag, str *value, int *tag_added )
{
	str *oldval;
	int n, status;

	if ( str_has_value( value ) ) {
		if ( *tag_added==1 ) {
			n = fields_num( risin );
			if ( n>0 ) {
				oldval = fields_value( risin, n-1, FIELDS_STRP );
				str_addchar( oldval, ' ' );
				str_strcat( oldval, value );
				if ( str_memerr( oldval ) ) return BIBL_ERR_MEMERR;
			}
		}
		else  {
			status = fields_add( risin, str_cstr( tag ), str_cstr( value ), 0 );
			if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
			*tag_added = 1;
		}
	}
	return BIBL_OK;
}

static int
add_tag_value( fields *risin, str *tag, str *value, int *tag_added )
{
	int status;

	if ( str_has_value( value ) ) {
		status = fields_add( risin, str_cstr( tag ), str_cstr( value ), 0 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		*tag_added = 1;
	}

	else {
		*tag_added = 0;
	}

	return BIBL_OK;
}

static int
risin_processf( fields *risin, const char *p, const char *filename, long nref, param *pm )
{
	int status, tag_added = 0, ret = 1;
	str tag, value;

	strs_init( &tag, &value, NULL );

	while ( *p ) {

		/* ...tag, add entry */
		if ( is_ris_tag( p ) ) {
			str_empty( &tag );
			str_empty( &value );
			p = process_tagged_line( &tag, &value, p );
			status = add_tag_value( risin, &tag, &value, &tag_added );
			if ( status!=BIBL_OK ) {
				ret = 0;
				goto out;
			}
		}

		/* ...no tag, merge with previous line */
		else {
			str_empty( &value );
			p = process_untagged_line( &value, p );
			status = merge_tag_value( risin, &tag, &value, &tag_added );
			if ( status!=BIBL_OK ) {
				ret = 0;
				goto out;
			}
		}

	}
out:

	strs_free( &tag, &value, NULL );
	return ret;
}

/*****************************************************
 PUBLIC: int risin_typef()
*****************************************************/

static int
risin_typef( fields *risin, const char *filename, int nref, param *p )
{
	int ntypename, nrefname, is_default;
	char *refname = "", *typename = "";

	ntypename = fields_find( risin, "TY", LEVEL_MAIN );
	nrefname  = fields_find( risin, "ID", LEVEL_MAIN );
	if ( ntypename!=FIELDS_NOTFOUND ) typename = fields_value( risin, ntypename, FIELDS_CHRP_NOUSE );
	if ( nrefname!=FIELDS_NOTFOUND )  refname  = fields_value( risin, nrefname,  FIELDS_CHRP_NOUSE );

	return get_reftype( typename, nref, p->progname, p->all, p->nall, refname, &is_default, REFTYPE_CHATTY );
}

/*****************************************************
 PUBLIC: int risin_convertf()
*****************************************************/

static int
is_uri_file_scheme( char *p )
{
	if ( !strncmp( p, "file:", 5 ) ) return 5;
	return 0;
}

static int
risin_linkedfile( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	int fstatus, m;
	char *p;

	/* if URL is file:///path/to/xyz.pdf, only store "///path/to/xyz.pdf" */
	m = is_uri_file_scheme( str_cstr( invalue ) );
	if ( m ) {
		/* skip past "file:" and store only actual path */
		p = invalue->data + m;
		fstatus = fields_add( bibout, outtag, p, level );
		if ( fstatus==FIELDS_OK ) return BIBL_OK;
		else return BIBL_ERR_MEMERR;
	}

	/* if URL is http:, ftp:, etc. store as a URL */
	m = is_uri_remote_scheme( str_cstr( invalue ) );
	if ( m!=-1 ) {
		fstatus = fields_add( bibout, "URL", str_cstr( invalue ), level );
		if ( fstatus==FIELDS_OK ) return BIBL_OK;
		else return BIBL_ERR_MEMERR;
	}

	/* badly formed, RIS wants URI, but store value anyway */
	fstatus = fields_add( bibout, outtag, str_cstr( invalue ), level );
	if ( fstatus==FIELDS_OK ) return BIBL_OK;
	else return BIBL_ERR_MEMERR;
}

/* scopus puts DOI in the DO or DI tag, but it needs cleaning */
static int
risin_doi( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	int fstatus, doi;
	doi = is_doi( str_cstr( invalue ) );
	if ( doi!=-1 ) {
		fstatus = fields_add( bibout, "DOI", &(invalue->data[doi]), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	return BIBL_OK;
}

static int
risin_date( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	char *p = str_cstr( invalue );
	int part, status;
	str date;

	part = ( !strncasecmp( outtag, "PART", 4 ) );

	str_init( &date );
	while ( *p && *p!='/' ) str_addchar( &date, *p++ );
	if ( str_memerr( &date ) ) return BIBL_ERR_MEMERR;
	if ( *p=='/' ) p++;
	if ( str_has_value( &date ) ) {
		if ( part ) status = fields_add( bibout, "PARTDATE:YEAR", str_cstr( &date ), level );
		else        status = fields_add( bibout, "DATE:YEAR",     str_cstr( &date ), level );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	str_empty( &date );
	while ( *p && *p!='/' ) str_addchar( &date, *p++ );
	if ( str_memerr( &date ) ) return BIBL_ERR_MEMERR;
	if ( *p=='/' ) p++;
	if ( str_has_value( &date ) ) {
		if ( part ) status = fields_add( bibout, "PARTDATE:MONTH", str_cstr( &date ), level );
		else        status = fields_add( bibout, "DATE:MONTH",     str_cstr( &date ), level );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	str_empty( &date );
	while ( *p && *p!='/' ) str_addchar( &date, *p++ );
	if ( str_memerr( &date ) ) return BIBL_ERR_MEMERR;
	if ( *p=='/' ) p++;
	if ( str_has_value( &date ) ) {
		if ( part ) status = fields_add( bibout, "PARTDATE:DAY", str_cstr( &date ), level );
		else        status = fields_add( bibout, "DATE:DAY",     str_cstr( &date ), level );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	str_empty( &date );
	while ( *p ) str_addchar( &date, *p++ );
	if ( str_memerr( &date ) ) return BIBL_ERR_MEMERR;
	if ( str_has_value( &date ) ) {
		if ( part ) status = fields_add( bibout, "PARTDATE:OTHER", str_cstr( &date ), level );
		else        status = fields_add( bibout, "DATE:OTHER",     str_cstr( &date ), level );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	str_free( &date );
	return BIBL_OK;
}

static int
risin_person( fields *bibin, int n, str *intag, str *invalue, int level, param *pm, char *outtag, fields *bibout )
{
	int i, begin, end, ok, status = BIBL_OK;
	slist tokens;
	str name;

	str_init( &name );
	slist_init( &tokens );

	status = slist_tokenize( &tokens, invalue, " \t\r\n", 1 );
	if ( status!=SLIST_OK ) { status = BIBL_ERR_MEMERR; goto out; }

	begin = 0;
	while ( begin < tokens.n ) {

		end = begin + 1;

		while ( end < tokens.n && strcasecmp( slist_cstr( &tokens, end ), "and" ) )
			end++;

		str_empty( &name );
		for ( i=begin; i<end; ++i ) {
			if ( i>begin ) str_addchar( &name, ' ' );
			str_strcat( &name, slist_str( &tokens, i ) );
		}

		ok = name_add( bibout, outtag, str_cstr( &name ), level, &(pm->asis), &(pm->corps) );
		if ( !ok ) { status = BIBL_ERR_MEMERR; goto out; }

		begin = end + 1;

		/* Handle repeated 'and' errors */
		while ( begin < tokens.n && !strcasecmp( slist_cstr( &tokens, begin ), "and" ) )
			begin++;

	}

out:
	str_free( &name );
	slist_free( &tokens );
	return status;
}

/* look for thesis-type hint */
static int
risin_thesis_hints( fields *bibin, int reftype, param *p, fields *bibout )
{
	int i, nfields, fstatus;
	char *tag, *value;

	if ( strcasecmp( p->all[reftype].type, "THES" ) ) return BIBL_OK;

	nfields = fields_num( bibin );
	for ( i=0; i<nfields; ++i ) {
		tag = fields_tag( bibin, i, FIELDS_CHRP );
		if ( strcasecmp( tag, "U1" ) ) continue;
		value = fields_value( bibin, i, FIELDS_CHRP );
		if ( !strcasecmp(value,"Ph.D. Thesis")||
		     !strcasecmp(value,"Masters Thesis")||
		     !strcasecmp(value,"Diploma Thesis")||
		     !strcasecmp(value,"Doctoral Thesis")||
		     !strcasecmp(value,"Habilitation Thesis")||
		     !strcasecmp(value,"Licentiate Thesis")) {
			fstatus = fields_add( bibout, "GENRE:BIBUTILS", value, 0 );
			if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
	}
	return BIBL_OK;
}

static void
risin_report_notag( param *p, char *tag )
{
	if ( p->verbose && strcmp( tag, "TY" ) ) {
		if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
		fprintf( stderr, "Did not identify RIS tag '%s'\n", tag );
	}
}

static int
risin_convertf( fields *bibin, fields *bibout, int reftype, param *p )
{
	static int (*convertfns[NUM_REFTYPES])(fields *, int, str *, str *, int, param *, char *, fields *) = {
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
		[ SERIALNO     ] = generic_serialno,
		[ NOTES        ] = generic_notes,
		[ URL          ] = generic_url,
		[ GENRE        ] = generic_genre,
		[ PERSON       ] = risin_person,
		[ DATE         ] = risin_date,
		[ DOI          ] = risin_doi,
		[ LINKEDFILE   ] = risin_linkedfile,
        };
	int process, level, i, nfields, status = BIBL_OK;
	str *intag, *invalue;
	char *outtag;

	nfields = fields_num( bibin );

	for ( i=0; i<nfields; ++i ) {
		intag = fields_tag( bibin, i, FIELDS_STRP );
		if ( !translate_oldtag( str_cstr( intag ), reftype, p->all, p->nall, &process, &level, &outtag ) ) {
			risin_report_notag( p, str_cstr( intag ) );
			continue;
		}
		invalue = fields_value( bibin, i, FIELDS_STRP );

		status = convertfns[ process ] ( bibin, i, intag, invalue, level, p, outtag, bibout );
		if ( status!=BIBL_OK ) return status;
	}

	if ( status == BIBL_OK ) status = risin_thesis_hints( bibin, reftype, p, bibout );

	if ( status==BIBL_OK && p->verbose ) fields_report( bibout, stderr );

	return status;
}
