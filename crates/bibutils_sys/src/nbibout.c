/*
 * nbibout.c
 *
 * Copyright (c) Chris Putnam 2018-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "utf8.h"
#include "str.h"
#include "is_ws.h"
#include "fields.h"
#include "generic.h"
#include "iso639_3.h"
#include "title.h"
#include "bibutils.h"
#include "bibformats.h"

/*****************************************************
 PUBLIC: int nbibout_initparams()
*****************************************************/

static int  nbibout_write( fields *info, FILE *fp, param *p, unsigned long refnum );

int
nbibout_initparams( param *pm, const char *progname )
{
	pm->writeformat      = BIBL_NBIBOUT;
	pm->format_opts      = 0;
	pm->charsetout       = BIBL_CHARSET_DEFAULT;
	pm->charsetout_src   = BIBL_SRC_DEFAULT;
	pm->latexout         = 0;
	pm->utf8out          = BIBL_CHARSET_UTF8_DEFAULT;
	pm->utf8bom          = BIBL_CHARSET_BOM_DEFAULT;
	pm->xmlout           = BIBL_XMLOUT_FALSE;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->singlerefperfile = 0;

	if ( pm->charsetout == BIBL_CHARSET_UNICODE ) {
		pm->utf8out = pm->utf8bom = 1;
	}

	pm->headerf = generic_writeheader;
	pm->footerf = NULL;
	pm->writef  = nbibout_write;

	if ( !pm->progname ) {
		if ( !progname ) pm->progname = NULL;
		else {
			pm->progname = strdup( progname );
			if ( !pm->progname ) return BIBL_ERR_MEMERR;
		}
	}

	return BIBL_OK;
}

/*****************************************************
 PUBLIC: int nbibout_write()
*****************************************************/

enum {
        TYPE_UNKNOWN = 0,
        TYPE_ARTICLE = 1,
        TYPE_INBOOK  = 2,
        TYPE_BOOK    = 3,
};

static void
append_type( fields *in, fields *out, int *status )
{
	int fstatus;
	char *s;
        int type = TYPE_UNKNOWN, i, n, level;
	char *tag, *value;

	n = fields_num( in );
        for ( i=0; i<n; ++i ) {
		tag = fields_tag( in, i, FIELDS_CHRP );
                if ( strcasecmp( tag, "GENRE:MARC" ) &&
		     strcasecmp( tag, "GENRE:BIBUTILS" ) &&
		     strcasecmp( tag, "GENRE:UNKNOWN" ) ) continue;
		value = fields_value( in, i, FIELDS_CHRP );
		level = fields_level( in, i );
                if ( !strcasecmp( value, "periodical" ) ||
                     !strcasecmp( value, "academic journal" ) ||
		     !strcasecmp( value, "journal article" ) ) {
                        type = TYPE_ARTICLE;
                } else if ( !strcasecmp( value, "book" ) ) {
                        if ( level==0 ) type=TYPE_BOOK;
                        else type=TYPE_INBOOK;
		} else if ( !strcasecmp( value, "book chapter" ) ) {
			type = TYPE_INBOOK;
                }
        }

	if ( type==TYPE_ARTICLE ) s = "Journal Article";
	else if ( type==TYPE_INBOOK ) s = "Chapter";
	else if ( type==TYPE_BOOK ) s = "Book";
	else s = "Miscellaneous";

	fstatus = fields_add( out, "PT", s, LEVEL_MAIN );
	if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
}

static void
append_titlecore( fields *in, char *nbibtag, int level, char *maintag, char *subtag, fields *out, int *status )
{
	str *mainttl = fields_findv( in, level, FIELDS_STRP, maintag );
	str *subttl  = fields_findv( in, level, FIELDS_STRP, subtag );
	str fullttl;
	int fstatus;

	str_init( &fullttl );
	title_combine( &fullttl, mainttl, subttl );

	if ( str_memerr( &fullttl ) ) {
		*status = BIBL_ERR_MEMERR;
		goto out;
	}

	if ( str_has_value( &fullttl ) ) {
		fstatus = fields_add( out, nbibtag, str_cstr( &fullttl ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
out:
	str_free( &fullttl );
}

static void
append_title( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	append_titlecore( in, nbibtag, level, "TITLE", "SUBTITLE", out, status );
}

static void
append_abbrtitle( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	append_titlecore( in, nbibtag, level, "SHORTTITLE", "SHORTSUBTITLE", out, status );
}

static void
process_person( str *person, char *name, int full )
{
	str family, given, suffix;
	char *p = name;

	str_empty( person );

	strs_init( &family, &given, &suffix, NULL );

	while ( *p && *p!='|' )
		str_addchar( &family, *p++ );

	if ( full ) {
		while ( *p=='|' && *(p+1)!='|' ) {
			p++;
			if ( *p!='|' && str_has_value( &given ) ) str_addchar( &given, ' ' );
			while ( *p && *p!='|' ) str_addchar( &given, *p++ );
		}
	}
	else {
		while ( *p=='|' && *(p+1)!='|' ) {
			p++;
			if ( *p!='|' ) str_addchar( &given, *p++ );
			while ( *p && *p!='|' ) p++;
		}
	}

	if ( *p=='|' && *(p+1)=='|' ) {
		p += 2;
		while ( *p && *p!='|' ) str_addchar( &suffix, *p++ );
	}

	if ( str_has_value( &family ) ) str_strcat( person, &family );
	if ( str_has_value( &suffix ) ) {
		if ( str_has_value( &family ) ) str_strcatc( person, " " );
		str_strcat( person, &suffix );
	}
	if ( str_has_value( &given ) ) {
		if ( str_has_value( person ) ) {
			if ( full ) str_strcatc( person, ", " );
			else        str_strcatc( person, " " );
		}
		str_strcat( person, &given );
	}

	strs_free( &family, &given, &suffix, NULL );
}

static void
append_people( fields *f, char *tag, char *nbibtag_full, char *nbibtag_abbr, int level, fields *out, int *status )
{
	vplist_index i;
	vplist people;
	str person;
	int fstatus;

	str_init( &person );
	vplist_init( &people );

	fields_findv_each( f, level, FIELDS_CHRP, &people, tag );
	for ( i=0; i<people.n; ++i ) {

		process_person( &person, (char *)vplist_get( &people, i ), 1 );
		if ( str_memerr( &person ) ) { *status = BIBL_ERR_MEMERR; goto out; }
		fstatus = fields_add_can_dup( out, nbibtag_full, str_cstr( &person ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) { *status = BIBL_ERR_MEMERR; goto out; }

		process_person( &person, (char *)vplist_get( &people, i ), 0 );
		if ( str_memerr( &person ) ) { *status = BIBL_ERR_MEMERR; goto out; }
		fstatus = fields_add_can_dup( out, nbibtag_abbr, str_cstr( &person ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) { *status = BIBL_ERR_MEMERR; goto out; }

	}

out:
	vplist_free( &people );
	str_free( &person );
}

static void
append_easy( fields *in, char *tag, char *nbibtag, int level, fields *out, int *status )
{
	char *value;
	int fstatus;

	value = fields_findv( in, level, FIELDS_CHRP, tag );
	if ( value ) {
		fstatus = fields_add( out, nbibtag, value, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
}

static void
append_easyall( fields *in, char *tag, char *nbibtag, int level, fields *out, int *status )
{
	vplist_index i;
	int fstatus;
	vplist a;

	vplist_init( &a );
	fields_findv_each( in, level, FIELDS_CHRP, &a, tag );
	for ( i=0; i<a.n; ++i ) {
		fstatus = fields_add( out, nbibtag, (char *) vplist_get( &a, i ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
	vplist_free( &a );
}

static void
append_pages( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	str *start, *stop, *articlenumber;
	int fstatus;
	str pages;
	char *p, *q;

	str_init( &pages );

	start = fields_findv_firstof( in, level, FIELDS_STRP, "PAGES:START", NULL );
	if ( start ) {
		str_strcpy( &pages, start );
	}

	stop  = fields_findv_firstof( in, level, FIELDS_STRP, "PAGES:STOP", NULL );
	if ( stop ) {
		/* nbib from pubmed doesn't do "PG - 101-109", but rather "PG - 101-9" */
		if ( start ) {
			p = str_cstr( start );
			q = str_cstr( stop );
			while ( *p && *p == *q ) {
				p++;
				q++;
			}
			if ( *q ) {
				str_addchar( &pages, '-' );
				str_strcatc( &pages, q );
			}
		}
		else {
			str_strcat( &pages, stop );
		}
	}

	articlenumber  = fields_findv_firstof( in, level, FIELDS_STRP, "ARTICLENUMBER", NULL );
	if ( str_is_empty( &pages ) && articlenumber ) {
		str_strcpy( &pages, articlenumber );
	}

	if ( str_has_value( &pages ) ) {
		fstatus = fields_add( out, nbibtag, str_cstr( &pages ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	str_free( &pages );
}

/* location identifier */
static void
append_lid( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	str *doi, *pii, *isi;
	int fstatus;
	str lid;

	str_init( &lid );

	doi = fields_findv( in, level, FIELDS_STRP, "DOI" );
	if ( doi ) {
		str_strcpy( &lid, doi );
		str_strcatc( &lid, " [doi]" );
		fstatus = fields_add( out, nbibtag, str_cstr( &lid ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	pii = fields_findv( in, level, FIELDS_STRP, "PII" );
	if ( pii ) {
		str_strcpy( &lid, pii );
		str_strcatc( &lid, " [pii]" );
		fstatus = fields_add( out, nbibtag, str_cstr( &lid ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	isi = fields_findv( in, level, FIELDS_STRP, "ISIREFNUM" );
	if ( isi ) {
		str_strcpy( &lid, isi );
		str_strcatc( &lid, " [isi]" );
		fstatus = fields_add( out, nbibtag, str_cstr( &lid ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}



	str_free( &lid );
}

static void
append_date( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	str *day, *month, *year;
	int fstatus;
	str date;

	str_init( &date );

	year  = fields_findv_firstof( in, level, FIELDS_STRP, "PARTDATE:YEAR",  "DATE:YEAR",  NULL );
	if ( year ) {
		str_strcpy( &date, year );
	}

	month = fields_findv_firstof( in, level, FIELDS_STRP, "PARTDATE:MONTH", "DATE:MONTH", NULL );
	if ( month ) {
		if ( str_has_value( &date ) ) str_addchar( &date, ' ' );
		str_strcat( &date, month );
	}

	day   = fields_findv_firstof( in, level, FIELDS_STRP, "PARTDATE:DAY",   "DATE:DAY",   NULL );
	if ( day ) {
		if ( str_has_value( &date ) ) str_addchar( &date, ' ' );
		str_strcat( &date, day );
	}

	if ( str_has_value( &date ) ) {
		fstatus = fields_add( out, nbibtag, str_cstr( &date ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
	
	str_free( &date );
}

static void
append_lang( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	int fstatus;
	str *lang;
	char *code;

	lang = fields_findv( in, level, FIELDS_STRP, "LANGUAGE" );
	if ( lang ) {
		code = iso639_3_from_name( str_cstr( lang ) );
		if ( !code ) code = str_cstr( lang );
		fstatus = fields_add( out, nbibtag, code, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
}

static void
append_keywords( fields *in, char *nbibtag, int level, fields *out, int *status )
{
	vplist keywords;
	int fstatus;
	char *kw;
	int i;

	vplist_init( &keywords );

	fields_findv_each( in, level, FIELDS_CHRP, &keywords, "KEYWORD" );
	for ( i=0; i<keywords.n; ++i ) {
		kw = vplist_get( &keywords, i );
		fstatus = fields_add( out, nbibtag, kw, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
	
}

static int
append_data( fields *in, fields *out )
{
	int status = BIBL_OK;

	append_easy ( in, "PMID",   "PMID", LEVEL_ANY,  out, &status );
	append_easyall( in, "ISSN", "IS",   LEVEL_ANY,  out, &status );
	append_easy ( in, "VOLUME", "VI",   LEVEL_ANY,  out, &status );
	append_easy ( in, "ISSUE",  "IP",   LEVEL_ANY,  out, &status );
	append_easy ( in, "NUMBER", "IP",   LEVEL_ANY,  out, &status );
	append_date ( in,           "DP",   LEVEL_ANY,  out, &status );
	append_title( in,           "TI",   LEVEL_MAIN, out, &status );
	append_pages( in,           "PG",   LEVEL_ANY,  out, &status );
	append_lid  ( in,           "LID",  LEVEL_MAIN, out, &status );
	append_easy ( in, "ABSTRACT", "AB", LEVEL_MAIN, out, &status );
	append_people ( in, "AUTHOR", "FAU",  "AU", LEVEL_MAIN, out, &status );
	append_easyall( in, "AUTHOR:CORP", "FAU", LEVEL_MAIN, out, &status );
	append_easyall( in, "AUTHOR:ASIS", "FAU", LEVEL_MAIN, out, &status );
	append_lang ( in,             "LA", LEVEL_ANY,  out, &status );
	append_type ( in, out, &status );
	append_easy(  in, "ADDRESS", "PL", LEVEL_MAIN, out, &status );
	append_abbrtitle( in, "TA", LEVEL_HOST, out, &status );
	append_title( in, "JT", LEVEL_HOST, out, &status );
	append_easy ( in, "PMC",    "PMC",  LEVEL_ANY,  out, &status );
	append_keywords( in, "OT", LEVEL_ANY, out, &status );
	return status;
}

static void
output_verbose( fields *f, const char *type, unsigned long refnum )
{
	char *tag, *value;
	int i, n, level;

	fprintf( stderr, "REF #%lu %s---\n", refnum+1, type );

	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		tag   = fields_tag( f, i, FIELDS_CHRP_NOUSE );
		value = fields_value( f, i, FIELDS_CHRP_NOUSE );
		level = fields_level( f, i );
		fprintf( stderr, "\t'%s'\t'%s'\t%d\n", tag, value, level );
	}

	fflush( stderr );
}

static void
output_tag( FILE *fp, char *p )
{
	int i = 0;

	while ( i < 4 && p && *p ) {
		fprintf( fp, "%c", *p );
		i++;
		p++;
	}

	for ( ; i<4; ++i )
		fprintf( fp, " " );
	fprintf( fp, "- " );
}

static void
output_value( FILE *fp, str *value )
{
	char *p, *q, *lastws;
	int n;

	if ( value->len < 82 ) {
		fprintf( fp, "%s", str_cstr( value ) );
		return;
	}

	p = str_cstr( value );
	while ( p && *p ) {
		n = 0;
		q = p;
		lastws = NULL;
		while ( n < 82 && *q ) {
			if ( is_ws( *q ) ) lastws = q;
			q++;
			n++;
		}
		if ( *q && lastws ) {
			while ( p!=lastws ) {
				fprintf( fp, "%c", *p );
				p++;
			}
			p++; /* skip ws separator */
		}
		else {
			while ( p!=q ) {
				fprintf( fp, "%c", *p );
				p++;
			}
			p = q;
		}
		if ( *p ) {
			fprintf( fp, "\n" );
			fprintf( fp, "      " );
		}
	}
}

static void
output_reference( FILE *fp, fields *out )
{
	int i;

	for ( i=0; i<out->n; ++i ) {

		output_tag( fp, ( char * ) fields_tag( out, i, FIELDS_CHRP ) );
		output_value( fp, ( str * ) fields_value( out, i, FIELDS_STRP ) );
		fprintf( fp, "\n" );
	}

        fprintf( fp, "\n\n" );
        fflush( fp );
}

static int
nbibout_write( fields *in, FILE *fp, param *p, unsigned long refnum )
{
	int status;
	fields out;

	fields_init( &out );

	if ( p->format_opts & BIBL_FORMAT_VERBOSE )
		output_verbose( in, "IN", refnum );

	status = append_data( in, &out );

	if ( status==BIBL_OK ) output_reference( fp, &out );

	if ( p->format_opts & BIBL_FORMAT_VERBOSE )
		output_verbose( &out, "OUT", refnum );

	fields_free( &out );

	return status;
}
