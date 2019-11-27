/*
 * risout.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "bibformats.h"
#include "fields.h"
#include "generic.h"
#include "name.h"
#include "str.h"
#include "title.h"
#include "url.h"
#include "utf8.h"

/*****************************************************
 PUBLIC: int risout_initparams()
*****************************************************/

static int  risout_write( fields *info, FILE *fp, param *p, unsigned long refnum );
static int  risout_assemble( fields *in, fields *out, param *pm, unsigned long refnum );

int
risout_initparams( param *pm, const char *progname )
{
	pm->writeformat      = BIBL_RISOUT;
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

	pm->headerf   = generic_writeheader;
	pm->footerf   = NULL;
	pm->assemblef = risout_assemble;
	pm->writef    = risout_write;

	if ( !pm->progname ) {
		if ( progname==NULL ) pm->progname = NULL;
		else {
			pm->progname = strdup( progname );
			if ( !pm->progname ) return BIBL_ERR_MEMERR;
		}
	}

	return BIBL_OK;
}

/*****************************************************
 PUBLIC: int risout_assemble()
*****************************************************/

enum { 
	TYPE_UNKNOWN = 0,
	TYPE_STD,                /* standard/generic (1) */
	TYPE_ABSTRACT,           /* abstract (2) */
	TYPE_ARTICLE,            /* article (3) */
	TYPE_BOOK,               /* book (4) */
	TYPE_CASE,               /* case (5) */
	TYPE_INBOOK,             /* chapter (6) */
	TYPE_CONF,               /* conference (7) */
	TYPE_ELEC,               /* electronic (8) */
	TYPE_HEAR,               /* hearing (9) */
	TYPE_MAGARTICLE,         /* magazine article (10) */
	TYPE_NEWSPAPER,          /* newspaper (11) */
	TYPE_MPCT,               /* mpct (12) */
	TYPE_PAMPHLET,           /* pamphlet (13) */
	TYPE_PATENT,             /* patent (14) */
	TYPE_PCOMM,              /* personal communication (15) */
	TYPE_PROGRAM,            /* program (16) */
	TYPE_REPORT,             /* report (17) */
	TYPE_STATUTE,            /* statute (18) */
	TYPE_THESIS,             /* thesis (19) */
	TYPE_LICENTIATETHESIS,   /* thesis (20) */
	TYPE_MASTERSTHESIS,      /* thesis (21) */
	TYPE_PHDTHESIS,          /* thesis (22) */
	TYPE_DIPLOMATHESIS,      /* thesis (23) */
	TYPE_DOCTORALTHESIS,     /* thesis (24) */
	TYPE_HABILITATIONTHESIS, /* thesis (25) */
	TYPE_MAP,                /* map, cartographic data (26) */
	TYPE_UNPUBLISHED,        /* unpublished (27) */
	NUM_TYPES 				 /* (28) */
};

static int type_is_element[ NUM_TYPES ] = {
	// Patch: Fix compilation on MSVC
	// [ 0 ... NUM_TYPES-1 ] = 0,
	[ 0     			] = 0,
	[ 1     			] = 0,
	[ 2      			] = 0,
	[ 3      			] = 0,
	[ 4     			] = 0,
	[ 5     			] = 0,
	[ 6      			] = 0,
	[ 7      			] = 0,
	[ 8     			] = 0,
	[ 9     			] = 0,
	[ 10      			] = 0,
	[ 11      			] = 0,
	[ 12    			] = 0,
	[ 13     			] = 0,
	[ 14      			] = 0,
	[ 15      			] = 0,
	[ 16     			] = 0,
	[ 17     			] = 0,
	[ 18      			] = 0,
	[ 19      			] = 0,
	[ 20     			] = 0,
	[ 21     			] = 0,
	[ 22      			] = 0,
	[ 23      			] = 0,
	[ 24     			] = 0,
	[ 25     			] = 0,
	[ 26      			] = 0,
	[ 27      			] = 0,
	[ TYPE_ARTICLE      ] = 1,
	[ TYPE_INBOOK       ] = 1,
	[ TYPE_MAGARTICLE   ] = 1,
	[ TYPE_NEWSPAPER    ] = 1,
	[ TYPE_ABSTRACT     ] = 1,
	[ TYPE_CONF         ] = 1,
};

static int type_uses_journal[ NUM_TYPES ] = {
	// Patch: Fix compilation on MSVC
	// [ 0 ... NUM_TYPES-1 ] = 0,
	[ 0     			] = 0,
	[ 1     			] = 0,
	[ 2      			] = 0,
	[ 3      			] = 0,
	[ 4     			] = 0,
	[ 5     			] = 0,
	[ 6      			] = 0,
	[ 7      			] = 0,
	[ 8     			] = 0,
	[ 9     			] = 0,
	[ 10      			] = 0,
	[ 11      			] = 0,
	[ 12    			] = 0,
	[ 13     			] = 0,
	[ 14      			] = 0,
	[ 15      			] = 0,
	[ 16     			] = 0,
	[ 17     			] = 0,
	[ 18      			] = 0,
	[ 19      			] = 0,
	[ 20     			] = 0,
	[ 21     			] = 0,
	[ 22      			] = 0,
	[ 23      			] = 0,
	[ 24     			] = 0,
	[ 25     			] = 0,
	[ 26      			] = 0,
	[ 27      			] = 0,
	[ TYPE_ARTICLE      ] = 1,
	[ TYPE_MAGARTICLE   ] = 1,
};

static void
write_type( FILE *fp, int type )
{
	const char *typenames[ NUM_TYPES ] = {
		[ TYPE_UNKNOWN            ] = "TYPE_UNKNOWN",
		[ TYPE_STD                ] = "TYPE_STD",
		[ TYPE_ABSTRACT           ] = "TYPE_ABSTRACT",
		[ TYPE_ARTICLE            ] = "TYPE_ARTICLE",
		[ TYPE_BOOK               ] = "TYPE_BOOK",
		[ TYPE_CASE               ] = "TYPE_CASE",
		[ TYPE_INBOOK             ] = "TYPE_INBOOK",
		[ TYPE_CONF               ] = "TYPE_CONF",
		[ TYPE_ELEC               ] = "TYPE_ELEC",
		[ TYPE_HEAR               ] = "TYPE_HEAR",
		[ TYPE_MAGARTICLE         ] = "TYPE_MAGARTICLE",
		[ TYPE_NEWSPAPER          ] = "TYPE_NEWSPAPER",
		[ TYPE_MPCT               ] = "TYPE_MPCT",
		[ TYPE_PAMPHLET           ] = "TYPE_PAMPHLET",
		[ TYPE_PATENT             ] = "TYPE_PATENT",
		[ TYPE_PCOMM              ] = "TYPE_PCOMM",
		[ TYPE_PROGRAM            ] = "TYPE_PROGRAM",
		[ TYPE_REPORT             ] = "TYPE_REPORT",
		[ TYPE_STATUTE            ] = "TYPE_STATUTE",
		[ TYPE_THESIS             ] = "TYPE_THESIS",
		[ TYPE_LICENTIATETHESIS   ] = "TYPE_LICENTIATETHESIS",
		[ TYPE_MASTERSTHESIS      ] = "TYPE_MASTERSTHESIS",
		[ TYPE_PHDTHESIS          ] = "TYPE_PHDTHESIS",
		[ TYPE_DIPLOMATHESIS      ] = "TYPE_DIPLOMATHESIS",
		[ TYPE_DOCTORALTHESIS     ] = "TYPE_DOCTORALTHESIS",
		[ TYPE_HABILITATIONTHESIS ] = "TYPE_HABILITATIONTHESIS",
		[ TYPE_MAP                ] = "TYPE_MAP",
		[ TYPE_UNPUBLISHED        ] = "TYPE_UNPUBLISHED",
	};

	if ( type < 0 || type >= NUM_TYPES ) fprintf( fp, "Error - type not in enum" );
	else fprintf( fp, "%s", typenames[ type ] );
}

static void
verbose_type_identified( char *element_type, param *p, int type )
{
	if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
	fprintf( stderr, "Type from %s element: ", element_type );
	write_type( stderr, type );
	fprintf( stderr, "\n" );
}

static void
verbose_type_assignment( char *tag, char *value, param *p, int type )
{
	if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
	fprintf( stderr, "Type from tag '%s' data '%s': ", tag, value );
	write_type( stderr, type );
	fprintf( stderr, "\n" );
}

typedef struct match_type {
	char *name;
	int type;
} match_type;

/* Try to determine type of reference from
 * <genre></genre>
 */
static int
get_type_genre( fields *f, param *p )
{
	match_type match_genres[] = {
		{ "academic journal",          TYPE_ARTICLE },
		{ "article",                   TYPE_ARTICLE },
		{ "journal article",           TYPE_ARTICLE },
		{ "magazine",                  TYPE_MAGARTICLE },
		{ "conference publication",    TYPE_CONF },
		{ "newspaper",                 TYPE_NEWSPAPER },
		{ "legislation",               TYPE_STATUTE },
		{ "communication",             TYPE_PCOMM },
		{ "hearing",                   TYPE_HEAR },
		{ "electronic",                TYPE_ELEC },
		{ "legal case and case notes", TYPE_CASE },
		{ "book chapter",              TYPE_INBOOK },
		{ "Ph.D. thesis",              TYPE_PHDTHESIS },
		{ "Licentiate thesis",         TYPE_LICENTIATETHESIS },
		{ "Masters thesis",            TYPE_MASTERSTHESIS },
		{ "Diploma thesis",            TYPE_DIPLOMATHESIS },
		{ "Doctoral thesis",           TYPE_DOCTORALTHESIS },
		{ "Habilitation thesis",       TYPE_HABILITATIONTHESIS },
		{ "report",                    TYPE_REPORT },
		{ "technical report",          TYPE_REPORT },
		{ "abstract or summary",       TYPE_ABSTRACT },
		{ "patent",                    TYPE_PATENT },
		{ "unpublished",               TYPE_UNPUBLISHED },
		{ "manuscript",                TYPE_UNPUBLISHED },
		{ "map",                       TYPE_MAP },
	};
	int nmatch_genres = sizeof( match_genres ) / sizeof( match_genres[0] );
	char *tag, *value;
	int type, i, j;

	type = TYPE_UNKNOWN;

	for ( i=0; i<fields_num( f ); ++i ) {
		tag = ( char * ) fields_tag( f, i, FIELDS_CHRP );
		if ( strcmp( tag, "GENRE:MARC" ) && strcmp( tag, "GENRE:BIBUTILS" ) && strcmp( tag, "GENRE:UNKNOWN") ) continue;
		value = ( char * ) fields_value( f, i, FIELDS_CHRP );
		for ( j=0; j<nmatch_genres; ++j )
			if ( !strcasecmp( match_genres[j].name, value ) )
				type = match_genres[j].type;
		if ( p->verbose ) verbose_type_assignment( tag, value, p, type );
		if ( type==TYPE_UNKNOWN ) {
			if ( !strcasecmp( value, "periodical" ) )
				type = TYPE_ARTICLE;
			else if ( !strcasecmp( value, "thesis" ) )
				type = TYPE_THESIS;
			else if ( !strcasecmp( value, "book" ) ) {
				if ( fields_level( f, i )==0 ) type=TYPE_BOOK;
				else type=TYPE_INBOOK;
			}
			else if ( !strcasecmp( value, "collection" ) ) {
				if ( fields_level( f, i )==0 ) type=TYPE_BOOK;
				else type=TYPE_INBOOK;
			}
		}

	}

	if ( p->verbose ) verbose_type_identified( "genre", p, type );

	return type;
}

/* Try to determine type of reference from
 * <TypeOfResource></TypeOfResource>
 */
static int
get_type_resource( fields *f, param *p )
{
	match_type match_res[] = {
		{ "software, multimedia",      TYPE_PROGRAM },
		{ "cartographic",              TYPE_MAP     },
	};
	int nmatch_res = sizeof( match_res ) / sizeof( match_res[0] );
	vplist_index i;
	int type, j;
	char *value;
	vplist a;

	type = TYPE_UNKNOWN;

	vplist_init( &a );
	fields_findv_each( f, LEVEL_ANY, FIELDS_CHRP, &a, "RESOURCE" );

	for ( i=0; i<a.n; ++i ) {
		value = ( char * ) vplist_get( &a, i );
		for ( j=0; j<nmatch_res; ++j ) {
			if ( !strcasecmp( value, match_res[j].name ) )
				type = match_res[j].type;
		}
		if ( p->verbose ) verbose_type_assignment( "RESOURCE", value, p, type );
	}

	if ( p->verbose ) verbose_type_identified( "resource", p, type );

	vplist_free( &a );
	return type;
}

/* Try to determine type of reference from <issuance></issuance> and */
/* <typeOfReference></typeOfReference> */
static int
get_type_issuance( fields *f, param *p )
{
	int type = TYPE_UNKNOWN;
	int i, monographic = 0, monographic_level = 0;
//	int text = 0;
	for ( i=0; i<f->n; ++i ) {
		if ( !strcasecmp( (char *) fields_tag( f, i, FIELDS_CHRP_NOUSE ), "issuance" ) &&
		     !strcasecmp( (char *) fields_value( f, i, FIELDS_CHRP_NOUSE ), "MONOGRAPHIC" ) ){
			monographic = 1;
			monographic_level = f->level[i];
		}
//		if ( !strcasecmp( (char *) fields_tag( f, i, FIELDS_CHRP_NOUSE ), "typeOfResource" ) &&
//		     !strcasecmp( (char *) fields_value( f, i, FIELDS_CHRP_NOUSE ), "text" ) ) {
//			text = 1;
//		}
	}
//	if ( monographic && text ) {
	if ( monographic ) {
		if ( monographic_level==0 ) type=TYPE_BOOK;
		else if ( monographic_level>0 ) type=TYPE_INBOOK;
	}

	if ( p->verbose ) {
		if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
		fprintf( stderr, "Type from issuance/typeOfReference elements: " );
		write_type( stderr, type );
		fprintf( stderr, "\n" );
	}

	return type;
}

static int
get_type( fields *f, param *p )
{
	int type;
	type = get_type_genre( f, p );
	if ( type==TYPE_UNKNOWN ) type = get_type_resource( f, p );
	if ( type==TYPE_UNKNOWN ) type = get_type_issuance( f, p );
	if ( type==TYPE_UNKNOWN ) {
		if ( fields_maxlevel( f ) > 0 ) type = TYPE_INBOOK;
		else type = TYPE_STD;
	}

	if ( p->verbose ) {
		if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
		fprintf( stderr, "Final type: " );
		write_type( stderr, type );
		fprintf( stderr, "\n" );
	}


	return type;
}

static void
append_type( int type, param *p, fields *out, int *status )
{
	char *typenames[ NUM_TYPES ] = {
		[ TYPE_STD                ] = "STD",
		[ TYPE_ABSTRACT           ] = "ABST",
		[ TYPE_ARTICLE            ] = "JOUR",
		[ TYPE_BOOK               ] = "BOOK",
		[ TYPE_CASE               ] = "CASE",
		[ TYPE_INBOOK             ] = "CHAP",
		[ TYPE_CONF               ] = "CONF",
		[ TYPE_ELEC               ] = "ELEC",
		[ TYPE_HEAR               ] = "HEAR",
		[ TYPE_MAGARTICLE         ] = "MGZN",
		[ TYPE_NEWSPAPER          ] = "NEWS",
		[ TYPE_MPCT               ] = "MPCT",
		[ TYPE_PAMPHLET           ] = "PAMP",
		[ TYPE_PATENT             ] = "PAT",
		[ TYPE_PCOMM              ] = "PCOMM",
		[ TYPE_PROGRAM            ] = "COMP",
		[ TYPE_REPORT             ] = "RPRT",
		[ TYPE_STATUTE            ] = "STAT",
		[ TYPE_THESIS             ] = "THES",
		[ TYPE_MASTERSTHESIS      ] = "THES",
		[ TYPE_PHDTHESIS          ] = "THES",
		[ TYPE_DIPLOMATHESIS      ] = "THES",
		[ TYPE_DOCTORALTHESIS     ] = "THES",
		[ TYPE_HABILITATIONTHESIS ] = "THES",
		[ TYPE_MAP                ] = "MAP",
		[ TYPE_UNPUBLISHED        ] = "UNPB",
	};
	int fstatus;

	if ( type < 0 || type >= NUM_TYPES ) {
		if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
		fprintf( stderr, "Internal error: Cannot recognize type %d, switching to TYPE_STD %d\n", type, TYPE_STD );
		type = TYPE_STD;
	}

	fstatus = fields_add( out, "TY", typenames[ type ], LEVEL_MAIN );
	if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
}

static void
append_people( fields *f, char *tag, char *ristag, int level, fields *out, int *status )
{
	vplist_index i;
	str oneperson;
	vplist people;
	int fstatus;

	str_init( &oneperson );
	vplist_init( &people );
	fields_findv_each( f, level, FIELDS_CHRP, &people, tag );
	for ( i=0; i<people.n; ++i ) {
		name_build_withcomma( &oneperson, ( char * ) vplist_get( &people, i ) );
		if ( str_memerr( &oneperson ) ) { *status = BIBL_ERR_MEMERR; goto out; }
		fstatus = fields_add_can_dup( out, ristag, str_cstr( &oneperson ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) { *status = BIBL_ERR_MEMERR; goto out; }
	}
out:
	vplist_free( &people );
	str_free( &oneperson );
}

static void
append_date( fields *in, fields *out, int *status )
{
	char *year, *month, *day;
	str date;
	int fstatus;

	year  = fields_findv_firstof( in, LEVEL_ANY, FIELDS_CHRP, "DATE:YEAR",  "PARTDATE:YEAR",  NULL );
	month = fields_findv_firstof( in, LEVEL_ANY, FIELDS_CHRP, "DATE:MONTH", "PARTDATE:MONTH", NULL );
	day   = fields_findv_firstof( in, LEVEL_ANY, FIELDS_CHRP, "DATE:DAY",   "PARTDATE:DAY",   NULL );

	if ( year ) {
		fstatus = fields_add( out, "PY", year, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	if ( year || month || day ) {
		str_init( &date );

		if ( year ) str_strcatc( &date, year );
		str_addchar( &date, '/' );
		if ( month ) str_strcatc( &date, month );
		str_addchar( &date, '/' );
		if ( day ) str_strcatc( &date, day );

		if ( str_memerr( &date ) ) { *status = BIBL_ERR_MEMERR; str_free( &date ); return; }

		fstatus = fields_add( out, "DA", str_cstr( &date ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;

		str_free( &date );
	}
}

static void
append_titlecore( fields *in, char *ristag, int level, char *maintag, char *subtag, fields *out, int *status )
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
		fstatus = fields_add( out, ristag, str_cstr( &fullttl ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

out:
	str_free( &fullttl );
}

static void
append_alltitles( fields *in, int type, fields *out, int *status )
{
	append_titlecore( in, "TI", 0, "TITLE", "SUBTITLE", out, status );
	append_titlecore( in, "T2", -1, "SHORTTITLE", "SHORTSUBTITLE", out, status );
	if ( type_is_element[ type ] ) {
		if ( type_uses_journal[ type ] )
			append_titlecore( in, "JO", 1, "TITLE", "SUBTITLE", out, status );
		else append_titlecore( in, "BT", 1, "TITLE", "SUBTITLE", out, status );
		append_titlecore( in, "T3", 2, "TITLE", "SUBTITLE", out, status );
	} else {
		append_titlecore( in, "T3", 1, "TITLE", "SUBTITLE", out, status );
	}
}

static void
append_pages( fields *in, fields *out, int *status )
{
	char *sn, *en, *ar;
	int fstatus;

	sn = fields_findv( in, LEVEL_ANY, FIELDS_CHRP, "PAGES:START" );
	en = fields_findv( in, LEVEL_ANY, FIELDS_CHRP, "PAGES:STOP" );

	if ( sn || en ) {
		if ( sn ) {
			fstatus = fields_add( out, "SP", sn, LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
		}
		if ( en ) {
			fstatus = fields_add( out, "EP", en, LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
		}
	} else {
		ar = fields_findv( in, LEVEL_ANY, FIELDS_CHRP, "ARTICLENUMBER" );
		if ( ar ) {
			fstatus = fields_add( out, "SP", ar, LEVEL_MAIN );
			if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
		}
	}
}

static void
append_keywords( fields *in, fields *out, int *status )
{
	vplist_index i;
	int fstatus;
	vplist vpl;

	vplist_init( &vpl );
	fields_findv_each( in, LEVEL_ANY, FIELDS_CHRP, &vpl, "KEYWORD" );
	for ( i=0; i<vpl.n; ++i ) {
		fstatus = fields_add( out, "KW", ( char * ) vplist_get( &vpl, i ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
	vplist_free( &vpl );
}

static void
append_urls( fields *in, fields *out, int *status )
{
	int lstatus;
	slist types;

	lstatus = slist_init_valuesc( &types, "URL", "DOI", "PMID", "PMC", "ARXIV", "JSTOR", "MRNUMBER", NULL );
	if ( lstatus!=SLIST_OK ) {
		*status = BIBL_ERR_MEMERR;
		return;
	}

	*status = urls_merge_and_add( in, LEVEL_ANY, out, "UR", LEVEL_MAIN, &types );

	slist_free( &types );

}

static void
append_thesishint( int type, fields *out, int *status )
{
	int fstatus;

	if ( type==TYPE_MASTERSTHESIS ) {
		fstatus = fields_add( out, "U1", "Masters thesis", LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	else if ( type==TYPE_PHDTHESIS ) {
		fstatus = fields_add( out, "U1", "Ph.D. thesis", LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	else if ( type==TYPE_DIPLOMATHESIS ) {
		fstatus = fields_add( out, "U1", "Diploma thesis", LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	else if ( type==TYPE_DOCTORALTHESIS ) {
		fstatus = fields_add( out, "U1", "Doctoral thesis", LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	else if ( type==TYPE_HABILITATIONTHESIS ) {
		fstatus = fields_add( out, "U1", "Habilitation thesis", LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	else if ( type==TYPE_LICENTIATETHESIS ) {
		fstatus = fields_add( out, "U1", "Licentiate thesis", LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
}

static int
is_uri_scheme( char *p )
{
	char *scheme[] = { "http:", "https:", "file:", "ftp:", "git:", "gopher:" };
	int i, len, nschemes = sizeof( scheme ) / sizeof( scheme[0] );
	for ( i=0; i<nschemes; ++i ) {
		len = strlen( scheme[i] );
		if ( !strncmp( p, scheme[i], len ) ) return len;
	}
	return 0;
}


static void
append_file( fields *in, char *tag, char *ristag, int level, fields *out, int *status )
{
	vplist_index i;
	str filename;
	int fstatus;
	vplist a;
	char *fl;

	str_init( &filename );
	vplist_init( &a );
	fields_findv_each( in, level, FIELDS_CHRP, &a, tag );
	for ( i=0; i<a.n; ++i ) {
		fl = ( char * ) vplist_get( &a, i );
		str_empty( &filename );
		if ( !is_uri_scheme( fl ) ) str_strcatc( &filename, "file:" );
		str_strcatc( &filename, fl );
		if ( str_memerr( &filename ) ) { *status = BIBL_ERR_MEMERR; goto out; }
		fstatus = fields_add( out, ristag, str_cstr( &filename ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) { *status = BIBL_ERR_MEMERR; goto out; }
	}
out:
	vplist_free( &a );
	str_free( &filename );
}

static void
append_easy( fields *in, char *tag, char *ristag, int level, fields *out, int *status )
{
	char *value;
	int fstatus;

	value = fields_findv( in, level, FIELDS_CHRP, tag );
	if ( value ) {
		fstatus = fields_add( out, ristag, value, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
}

static void
append_easyall( fields *in, char *tag, char *ristag, int level, fields *out, int *status )
{
	vplist_index i;
	int fstatus;
	vplist a;

	vplist_init( &a );
	fields_findv_each( in, level, FIELDS_CHRP, &a, tag );
	for ( i=0; i<a.n; ++i ) {
		fstatus = fields_add( out, ristag, (char *) vplist_get( &a, i ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
	vplist_free( &a );
}

static void
append_allpeople( fields *in, int type, fields *out, int *status )
{
	append_people ( in, "AUTHOR",      "AU", LEVEL_MAIN,   out, status );
	append_easyall( in, "AUTHOR:CORP", "AU", LEVEL_MAIN,   out, status );
	append_easyall( in, "AUTHOR:ASIS", "AU", LEVEL_MAIN,   out, status );
	append_people ( in, "AUTHOR",      "A2", LEVEL_HOST,   out, status );
	append_easyall( in, "AUTHOR:CORP", "A2", LEVEL_HOST,   out, status );
	append_easyall( in, "AUTHOR:ASIS", "A2", LEVEL_HOST,   out, status );
	append_people ( in, "AUTHOR",      "A3", LEVEL_SERIES, out, status );
	append_easyall( in, "AUTHOR:CORP", "A3", LEVEL_SERIES, out, status );
	append_easyall( in, "AUTHOR:ASIS", "A3", LEVEL_SERIES, out, status );
	append_people ( in, "EDITOR",      "ED", LEVEL_MAIN,   out, status );
	append_easyall( in, "EDITOR:CORP", "ED", LEVEL_MAIN,   out, status );
	append_easyall( in, "EDITOR:ASIS", "ED", LEVEL_MAIN,   out, status );
	if ( type_is_element[ type ] ) {
		append_people ( in, "EDITOR",      "ED", LEVEL_HOST, out, status );
		append_easyall( in, "EDITOR:CORP", "ED", LEVEL_HOST, out, status );
		append_easyall( in, "EDITOR:ASIS", "ED", LEVEL_HOST, out, status );
	} else {
		append_people ( in, "EDITOR",      "A3", LEVEL_HOST, out, status );
		append_easyall( in, "EDITOR:CORP", "A3", LEVEL_HOST, out, status );
		append_easyall( in, "EDITOR:ASIS", "A3", LEVEL_HOST, out, status );
	}
	append_people ( in, "EDITOR",      "A3", LEVEL_SERIES, out, status );
	append_easyall( in, "EDITOR:CORP", "A3", LEVEL_SERIES, out, status );
	append_easyall( in, "EDITOR:ASIS", "A3", LEVEL_SERIES, out, status );
}

static int
risout_assemble( fields *in, fields *out, param *pm, unsigned long refnum )
{
	int type, status = BIBL_OK;

	type = get_type( in, pm );

	append_type      ( type, pm, out, &status );
	append_allpeople ( in, type, out, &status );
	append_date      ( in, out, &status );
	append_alltitles ( in, type, out, &status );
	append_pages     ( in, out, &status );
	append_easy      ( in, "VOLUME",             "VL", LEVEL_ANY, out, &status );
	append_easy      ( in, "ISSUE",              "IS", LEVEL_ANY, out, &status );
	append_easy      ( in, "NUMBER",             "IS", LEVEL_ANY, out, &status );
	append_easy      ( in, "EDITION",            "ET", LEVEL_ANY, out, &status );
	append_easy      ( in, "NUMVOLUMES",         "NV", LEVEL_ANY, out, &status );
	append_easy      ( in, "ADDRESS:AUTHOR",     "AD", LEVEL_ANY, out, &status );
	append_easy      ( in, "PUBLISHER",          "PB", LEVEL_ANY, out, &status );
	append_easy      ( in, "DEGREEGRANTOR",      "PB", LEVEL_ANY, out, &status );
	append_easy      ( in, "DEGREEGRANTOR:ASIS", "PB", LEVEL_ANY, out, &status );
	append_easy      ( in, "DEGREEGRANTOR:CORP", "PB", LEVEL_ANY, out, &status );
	append_easy      ( in, "ADDRESS",            "CY", LEVEL_ANY, out, &status );
	append_keywords  ( in, out, &status );
	append_easy      ( in, "ABSTRACT",           "AB", LEVEL_ANY, out, &status );
	append_easy      ( in, "CALLNUMBER",         "CN", LEVEL_ANY, out, &status );
	append_easy      ( in, "ISSN",               "SN", LEVEL_ANY, out, &status );
	append_easy      ( in, "ISBN",               "SN", LEVEL_ANY, out, &status );
	append_file      ( in, "FILEATTACH",         "L1", LEVEL_ANY, out, &status );
	append_file      ( in, "FIGATTACH",          "L4", LEVEL_ANY, out, &status );
	append_easy      ( in, "CAPTION",            "CA", LEVEL_ANY, out, &status );
	append_urls      ( in, out, &status );
	append_easyall   ( in, "DOI",                "DO", LEVEL_ANY, out, &status );
	append_easy      ( in, "LANGUAGE",           "LA", LEVEL_ANY, out, &status );
	append_easy      ( in, "NOTES",              "N1", LEVEL_ANY, out, &status );
	append_easy      ( in, "REFNUM",             "ID", LEVEL_ANY, out, &status );
	append_thesishint( type, out, &status );

	return status;
}

/*****************************************************
 PUBLIC: int risout_write()
*****************************************************/

static int
risout_write( fields *out, FILE *fp, param *p, unsigned long refnum )
{
	const char *tag, *value;
	int i;

	for ( i=0; i<out->n; ++i ) {
		tag   = fields_tag  ( out, i, FIELDS_CHRP );
		value = fields_value( out, i, FIELDS_CHRP );
		fprintf( fp, "%s  - %s\n", tag, value );
	}

	fprintf( fp, "ER  - \n" );
	fflush( fp );
	return BIBL_OK;
}
