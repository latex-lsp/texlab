/*
 * adsout.c
 *
 * Copyright (c) Richard Mathar 2007-2019
 * Copyright (c) Chris Putnam 2007-2019
 *
 * Program and source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <ctype.h>
#include "utf8.h"
#include "str.h"
#include "strsearch.h"
#include "fields.h"
#include "generic.h"
#include "name.h"
#include "title.h"
#include "type.h"
#include "url.h"
#include "bibformats.h"

/*****************************************************
 PUBLIC: int adsout_initparams()
*****************************************************/

static int adsout_write( fields *in, FILE *fp, param *p, unsigned long refnum );
static int adsout_assemble( fields *in, fields *out, param *pm, unsigned long refnum );

int
adsout_initparams( param *pm, const char *progname )
{
	pm->writeformat      = BIBL_ADSABSOUT;
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
	pm->assemblef = adsout_assemble;
	pm->writef    = adsout_write;

	if ( !pm->progname ) {
		if ( !progname ) pm->progname=NULL;
		else {
			pm->progname = strdup( progname );
			if ( !pm->progname ) return BIBL_ERR_MEMERR;
		}
	}

	return BIBL_OK;
}

/*****************************************************
 PUBLIC: int adsout_assemble()
*****************************************************/

enum {
	TYPE_UNKNOWN = 0,
	TYPE_GENERIC,
	TYPE_ARTICLE,
	TYPE_MAGARTICLE,
	TYPE_BOOK,
	TYPE_INBOOK,
	TYPE_INPROCEEDINGS,
	TYPE_HEARING,
	TYPE_BILL,
	TYPE_CASE,
	TYPE_NEWSPAPER,
	TYPE_COMMUNICATION,
	TYPE_BROADCAST,
	TYPE_MANUSCRIPT,
	TYPE_REPORT,
	TYPE_THESIS,
	TYPE_MASTERSTHESIS,
	TYPE_PHDTHESIS,
	TYPE_DIPLOMATHESIS,
	TYPE_DOCTORALTHESIS,
	TYPE_HABILITATIONTHESIS,
	TYPE_LICENTIATETHESIS,
	TYPE_PATENT,
	TYPE_PROGRAM
};

static int
get_type( fields *in )
{
	match_type genre_matches[] = {
		{ "academic journal",          TYPE_ARTICLE,            LEVEL_ANY },
		{ "communication",             TYPE_COMMUNICATION,      LEVEL_ANY },
		{ "conference publication",    TYPE_INPROCEEDINGS,      LEVEL_ANY },
		{ "Diploma thesis",            TYPE_DIPLOMATHESIS,      LEVEL_ANY },
		{ "Doctoral thesis",           TYPE_DOCTORALTHESIS,     LEVEL_ANY },
		{ "Habilitation thesis",       TYPE_HABILITATIONTHESIS, LEVEL_ANY },
		{ "hearing",                   TYPE_HEARING,            LEVEL_ANY },
		{ "legal case and case notes", TYPE_CASE,               LEVEL_ANY },
		{ "legislation",               TYPE_BILL,               LEVEL_ANY },
		{ "Licentiate thesis",         TYPE_LICENTIATETHESIS,   LEVEL_ANY },
		{ "magazine",                  TYPE_MAGARTICLE,         LEVEL_ANY },
		{ "manuscript",                TYPE_MANUSCRIPT,         LEVEL_ANY },
		{ "Masters thesis",            TYPE_MASTERSTHESIS,      LEVEL_ANY },
		{ "newspaper",                 TYPE_NEWSPAPER,          LEVEL_ANY },
		{ "patent",                    TYPE_PATENT,             LEVEL_ANY },
		{ "Ph.D. thesis",              TYPE_PHDTHESIS,          LEVEL_ANY },
		{ "report",                    TYPE_REPORT,             LEVEL_ANY },
		{ "technical report",          TYPE_REPORT,             LEVEL_ANY },
		{ "unpublished",               TYPE_MANUSCRIPT,         LEVEL_ANY },
		{ "periodical",                TYPE_ARTICLE,            LEVEL_ANY  },
		{ "thesis",                    TYPE_THESIS,             LEVEL_ANY  },
		{ "book",                      TYPE_BOOK,               LEVEL_MAIN },
		{ "book",                      TYPE_INBOOK,             LEVEL_ANY  },
		{ "collection",                TYPE_BOOK,               LEVEL_MAIN },
		{ "collection",                TYPE_INBOOK,             LEVEL_ANY  },
	};
	int ngenre_matches = sizeof( genre_matches ) / sizeof( genre_matches[0] );

	match_type resource_matches[] = {
		{ "moving image",              TYPE_BROADCAST,          LEVEL_ANY  },
		{ "software, multimedia",      TYPE_PROGRAM,            LEVEL_ANY  },
	};
	int nresource_matches = sizeof( resource_matches ) /sizeof( resource_matches[0] );

	match_type issuance_matches[] = {
		{ "monographic",               TYPE_BOOK,               LEVEL_MAIN },
		{ "monographic",               TYPE_INBOOK,             LEVEL_ANY  },
	};
	int nissuance_matches = sizeof( issuance_matches ) /sizeof( issuance_matches[0] );

	int type;

	type = type_from_mods_hints( in, TYPE_FROM_GENRE, genre_matches, ngenre_matches, TYPE_UNKNOWN );
	if ( type==TYPE_UNKNOWN ) type = type_from_mods_hints( in, TYPE_FROM_RESOURCE, resource_matches, nresource_matches, TYPE_UNKNOWN );
	if ( type==TYPE_UNKNOWN ) type = type_from_mods_hints( in, TYPE_FROM_ISSUANCE, issuance_matches, nissuance_matches, TYPE_UNKNOWN );
	if ( type==TYPE_UNKNOWN ) type = TYPE_GENERIC;

	return type;
}

static int
append_title( fields *in, char *ttl, char *sub, char *adstag, int level, fields *out, int *status )
{
	str fulltitle, *title, *subtitle, *vol, *iss, *sn, *en, *ar;
	int fstatus, output = 0;

	str_init( &fulltitle );

	title     = fields_findv( in, level, FIELDS_STRP, ttl );
	subtitle  = fields_findv( in, level, FIELDS_STRP, sub );

	if ( str_has_value( title ) ) {

		output = 1;

		title_combine( &fulltitle, title, subtitle );

		vol = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "VOLUME" );
		if ( str_has_value( vol ) ) {
			str_strcatc( &fulltitle, ", vol. " );
			str_strcat( &fulltitle, vol );
		}

		iss = fields_findv_firstof( in, LEVEL_ANY, FIELDS_STRP, "ISSUE",
			"NUMBER", NULL );
		if ( str_has_value( iss ) ) {
			str_strcatc( &fulltitle, ", no. " );
			str_strcat( &fulltitle, iss );
		}

		sn = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "PAGES:START" );
		en = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "PAGES:STOP" );
		ar = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "ARTICLENUMBER" );

		if ( str_has_value( sn ) ) {
			if ( str_has_value( en ) ) {
				str_strcatc( &fulltitle, ", pp. " );
			} else {
				str_strcatc( &fulltitle, ", p. " );
			}
			str_strcat( &fulltitle, sn );
		} else if ( str_has_value( ar ) ) {
			str_strcatc( &fulltitle, ", p. " );
			str_strcat( &fulltitle, ar );
		}
		if ( str_has_value( en ) ) {
			str_addchar( &fulltitle, '-' );
			str_strcat( &fulltitle, en );
		}

		if ( str_memerr( &fulltitle ) ) {
			*status = BIBL_ERR_MEMERR;
			goto out;
		}

		fstatus = fields_add( out, adstag, str_cstr( &fulltitle ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;

	}

out:
	str_free( &fulltitle );

	return output;
}

static void
append_titles( fields *in, int type, fields *out, int *status )
{
	int added;
	if ( type==TYPE_ARTICLE || type==TYPE_MAGARTICLE ) {
		added = append_title( in, "TITLE", "SUBTITLE", "%J", LEVEL_HOST, out, status );
		if ( added==0 )
			(void) append_title( in, "SHORTTITLE", "SHORTSUBTITLE", "%J", LEVEL_HOST, out, status );
	}
}

static void
append_people( fields *in, char *tag1, char *tag2, char *tag3, char *adstag, int level, fields *out, int *status )
{
	str oneperson, allpeople;
	vplist_index i;
	int fstatus;
	vplist a;

	str_init( &oneperson );
	str_init( &allpeople );
	vplist_init( &a );

	fields_findv_eachof( in, level, FIELDS_CHRP, &a, tag1, tag2, tag3, NULL );
	if ( a.n ) {
		for ( i=0; i<a.n; ++i ) {
			if ( i!=0 ) str_strcatc( &allpeople, "; " );
			name_build_withcomma( &oneperson, (char *) vplist_get( &a, i) );
			str_strcat( &allpeople, &oneperson );
		}
		fstatus = fields_add( out, adstag, str_cstr( &allpeople ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	vplist_free( &a );
	str_free( &oneperson );
	str_free( &allpeople );
}

static void
append_pages( fields *in, fields *out, int *status )
{
	str *sn, *en, *ar;
	int fstatus;

	sn = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "PAGES:START" );
	en = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "PAGES:STOP" );
	ar = fields_findv( in, LEVEL_ANY, FIELDS_STRP, "ARTICLENUMBER" );

	if ( str_has_value( sn ) ) {
		fstatus = fields_add( out, "%P", str_cstr( sn ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) {
			*status = BIBL_ERR_MEMERR;
			return;
		}
	}

	else if ( str_has_value( ar ) ) {
		fstatus = fields_add( out, "%P", str_cstr( ar ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) {
			*status = BIBL_ERR_MEMERR;
			return;
		}
	}

	if ( str_has_value( en ) ) {
		fstatus = fields_add( out, "%L", str_cstr( en ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) {
			*status = BIBL_ERR_MEMERR;
			return;
		}
	}
}

static int
mont2mont( const char *m )
{
	static char *monNames[]= { "jan", "feb", "mar", "apr", "may", 
			"jun", "jul", "aug", "sep", "oct", "nov", "dec" };
	int i;
	if ( isdigit( (unsigned char)m[0] ) ) return atoi( m );
        else {
		for ( i=0; i<12; i++ ) {
			if ( !strncasecmp( m, monNames[i], 3 ) ) return i+1;
		}
	}
        return 0;
}

static int
get_month( fields *in, int level )
{
	str *month;

	month = fields_findv_firstof( in, level, FIELDS_STRP, "DATE:MONTH", "PARTDATE:MONTH", NULL );
	if ( str_has_value( month ) ) return mont2mont( str_cstr( month ) );
	else return 0;
}

static void
append_date( fields *in, char *adstag, int level, fields *out, int *status )
{
	int month, fstatus;
	char outstr[1000];
	str *year;

	year = fields_findv_firstof( in, level, FIELDS_STRP, "DATE:YEAR", "PARTDATE:YEAR", NULL );
	if ( str_has_value( year ) ) {
		month = get_month( in, level );
		sprintf( outstr, "%02d/%s", month, str_cstr( year ) );
		fstatus = fields_add( out, adstag, outstr, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
}

#include "adsout_journals.c"

static void
output_4digit_value( char *pos, long long n )
{
	char buf[6];
	n = n % 10000; /* truncate to 0->9999, will fit in buf[6] */
#ifdef WIN32
	sprintf( buf, "%I64d", n );
#else
	sprintf( buf, "%lld", n );
#endif
	if ( n < 10 )        strncpy( pos+3, buf, 1 );
	else if ( n < 100 )  strncpy( pos+2, buf, 2 );
	else if ( n < 1000 ) strncpy( pos+1, buf, 3 );
	else                 strncpy( pos,   buf, 4 );
}

static char
initial_ascii( const char *name )
{
	int b1, b2;

	if ( isascii( name[0] )  )
		return name[0];

        b1 = name[0]+256;
        b2 = name[1]+256;

	switch( b1 ) {

	case 0xc3:
		     if ( b2 >= 0x80 && b2 <= 0x86 ) return 'A';
		else if ( b2 == 0x87 )               return 'C';
		else if ( b2 >= 0x88 && b2 <= 0x8b ) return 'E';
		else if ( b2 >= 0x8c && b2 <= 0x8f ) return 'I';
		else if ( b2 == 0x90 )               return 'D';
		else if ( b2 == 0x91 )               return 'N';
		else if ( b2 >= 0x92 && b2 <= 0x98 ) return 'O';
		else if ( b2 >= 0x99 && b2 <= 0x9c ) return 'U';
		else if ( b2 == 0x9d )               return 'Y';
		else if ( b2 == 0x9f )               return 'S';
		else if ( b2 >= 0xa0 && b2 <= 0xa6 ) return 'A';
		else if ( b2 == 0xa7 )               return 'C';
		else if ( b2 >= 0xa8 && b2 <= 0xab ) return 'E';
		else if ( b2 >= 0xac && b2 <= 0xaf ) return 'I';
		else if ( b2 == 0xb0 )               return 'D';
		else if ( b2 == 0xb1 )               return 'N';
		else if ( b2 >= 0xb2 && b2 <= 0xb8 ) return 'O';
		else if ( b2 >= 0xb9 && b2 <= 0xbc ) return 'U';
		else if ( b2 >= 0xbd && b2 <= 0xbf ) return 'Y';
	break;

	case 0xc4:
		     if ( b2 >= 0x80 && b2 <= 0x85 ) return 'A';
		else if ( b2 >= 0x86 && b2 <= 0x8d ) return 'C';
		else if ( b2 >= 0x8e || b2 <= 0x91 ) return 'D';
		else if ( b2 >= 0x92 && b2 <= 0x9b ) return 'E';
		else if ( b2 >= 0x9c && b2 <= 0xa3 ) return 'G';
		else if ( b2 >= 0xa4 && b2 <= 0xa7 ) return 'H';
		else if ( b2 >= 0xa8 && b2 <= 0xb3 ) return 'I';
		else if ( b2 >= 0xb4 && b2 <= 0xb5 ) return 'J';
		else if ( b2 >= 0xb6 && b2 <= 0xb8 ) return 'K';
		else if ( b2 >= 0xb9 && b2 <= 0xbf ) return 'L';
	break;

	case 0xc5:
		     if ( b2 >= 0x80 && b2 <= 0x82 ) return 'L';
		else if ( b2 >= 0x83 && b2 <= 0x8b ) return 'N';
		else if ( b2 >= 0x8c || b2 <= 0x93 ) return 'O';
		else if ( b2 >= 0x94 && b2 <= 0x99 ) return 'R';
		else if ( b2 >= 0x9a && b2 <= 0xa1 ) return 'S';
		else if ( b2 >= 0xa2 && b2 <= 0xa7 ) return 'T';
		else if ( b2 >= 0xa8 && b2 <= 0xb3 ) return 'U';
		else if ( b2 >= 0xb4 && b2 <= 0xb5 ) return 'W';
		else if ( b2 >= 0xb6 && b2 <= 0xb8 ) return 'Y';
		else if ( b2 >= 0xb9 && b2 <= 0xbf ) return 'Z';
	break;

	case 0xc6:
		     if ( b2 >= 0x80 && b2 <= 0x85 ) return 'B';
		else if ( b2 >= 0x86 && b2 <= 0x88 ) return 'C';
		else if ( b2 >= 0x89 || b2 <= 0x8d ) return 'D';
		else if ( b2 >= 0x8e && b2 <= 0x90 ) return 'E';
		else if ( b2 >= 0x91 && b2 <= 0x92 ) return 'F';
		else if ( b2 >= 0x93 && b2 <= 0x94 ) return 'G';
		else if ( b2 == 0x95 )               return 'H';
		else if ( b2 >= 0x96 && b2 <= 0x97 ) return 'I';
		else if ( b2 >= 0x98 && b2 <= 0x99 ) return 'K';
		else if ( b2 >= 0xba && b2 <= 0x9b ) return 'L';
		else if ( b2 == 0xbc )               return 'M';
		else if ( b2 >= 0x9d && b2 <= 0x9e ) return 'N';
		else if ( b2 >= 0x9f && b2 <= 0xa3 ) return 'O';
		else if ( b2 >= 0xa4 && b2 <= 0xa5 ) return 'P';
		else if ( b2 == 0xa6 )               return 'R';
		else if ( b2 >= 0xa7 && b2 <= 0xaa ) return 'S';
		else if ( b2 >= 0xab && b2 <= 0xae ) return 'T';
		else if ( b2 >= 0xaf && b2 <= 0xb1 ) return 'U';
		else if ( b2 == 0xb2 )               return 'V';
		else if ( b2 >= 0xb3 && b2 <= 0xb4 ) return 'Y';
		else if ( b2 >= 0xb5 && b2 <= 0xbe ) return 'Z';
	break;

	}

	return '.';
}

static char
get_firstinitial( fields *in )
{
	char *name;
	int n;

	n = fields_find( in, "AUTHOR", LEVEL_MAIN );
	if ( n==FIELDS_NOTFOUND ) n = fields_find( in, "AUTHOR", LEVEL_ANY );

	if ( n!=FIELDS_NOTFOUND ) {
		name = fields_value( in, n, FIELDS_CHRP );
		return initial_ascii( name );
	} else return '\0';
}

static int
get_journalabbr( fields *in )
{
	char *jrnl;
	int n, j;

	n = fields_find( in, "TITLE", LEVEL_HOST );
	if ( n!=FIELDS_NOTFOUND ) {
		jrnl = fields_value( in, n, FIELDS_CHRP );
		for ( j=0; j<njournals; j++ ) {
			if ( !strcasecmp( jrnl, journals[j]+6 ) )
				return j;
		}
	}
	return -1;
}

static void
append_Rtag( fields *in, char *adstag, int type, fields *out, int *status )
{
	char outstr[20], ch;
	int n, i, fstatus;
	long long page;

	strcpy( outstr, "..................." );

	/** YYYY */
	n = fields_find( in, "DATE:YEAR", LEVEL_ANY );
	if ( n==FIELDS_NOTFOUND ) n = fields_find( in, "PARTDATE:YEAR", LEVEL_ANY );
	if ( n!=FIELDS_NOTFOUND ) output_4digit_value( outstr, atoi( fields_value( in, n, FIELDS_CHRP ) ) );

	/** JJJJ */
	n = get_journalabbr( in );
	if ( n!=-1 ) {
		i = 0;
		while ( i<5 && journals[n][i]!=' ' && journals[n][i]!='\t' ) {
			outstr[4+i] = journals[n][i];
			i++;
		}
	}

	/** VVVV */
	n = fields_find( in, "VOLUME", LEVEL_ANY );
	if ( n!=FIELDS_NOTFOUND ) output_4digit_value( outstr+9, atoi( fields_value( in, n, FIELDS_CHRP ) ) );

	/** MPPPP */
	n = fields_find( in, "PAGES:START", LEVEL_ANY );
	if ( n==FIELDS_NOTFOUND ) n = fields_find( in, "ARTICLENUMBER", LEVEL_ANY );
	if ( n!=FIELDS_NOTFOUND ) {
		page = atoll( fields_value( in, n, FIELDS_CHRP ) );
		output_4digit_value( outstr+14, page );
		if ( page>=10000 ) {
			ch = 'a' + (page/10000);
			outstr[13] = ch;
		}
	}

	/** A */
	 ch = toupper( (unsigned char) get_firstinitial( in ) );
	if ( ch!='\0' ) outstr[18] = ch;

	fstatus = fields_add( out, adstag, outstr, LEVEL_MAIN );
	if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
}

static void
append_easyall( fields *in, char *tag, char *adstag, int level, fields *out, char *prefix, int *status )
{
	vplist_index i;
	int fstatus;
	str output;
	char *val;
	vplist a;

	vplist_init( &a );
	if ( prefix ) str_init( &output );

	fields_findv_each( in, level, FIELDS_CHRP, &a, tag );

	for ( i=0; i<a.n; ++i ) {
		val = ( char * ) vplist_get( &a, i );
		if ( prefix ) {
			str_strcpyc( &output, prefix );
			str_strcatc( &output, val );
			val = str_cstr( &output );
		}
		fstatus = fields_add( out, adstag, val, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) {
			*status = BIBL_ERR_MEMERR;
			goto out;
		}
	}
out:
	if ( prefix ) str_free( &output );
	vplist_free( &a );
}

static void
append_easy( fields *in, char *tag, char *adstag, int level, fields *out, int *status )
{
	char *value;
	int fstatus;

	value = fields_findv( in, level, FIELDS_CHRP, tag );
	if ( value && value[0]!='\0' ) {
		fstatus = fields_add( out, adstag, value, LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}
}

static void
append_keys( fields *in, char *tag, char *adstag, int level, fields *out, int *status )
{
	vplist_index i;
	str allkeys;
	int fstatus;
	vplist a;

	str_init( &allkeys );
	vplist_init( &a );

	fields_findv_each( in, level, FIELDS_CHRP, &a, tag );

	if ( a.n ) {
		for ( i=0; i<a.n; ++i ) {
			if ( i>0 ) str_strcatc( &allkeys, ", " );
			str_strcatc( &allkeys, (char *) vplist_get( &a, i ) );
		}
		fstatus = fields_add( out, adstag, str_cstr( &allkeys ), LEVEL_MAIN );
		if ( fstatus!=FIELDS_OK ) *status = BIBL_ERR_MEMERR;
	}

	str_free( &allkeys );
	vplist_free( &a );
}

static void
append_urls( fields *in, fields *out, int *status )
{
	int lstatus;
	slist types;

	/* skip DOI as we'll add that separately */
	lstatus = slist_init_valuesc( &types, "URL", "PMID", "PMC", "ARXIV", "JSTOR", "MRNUMBER", "FILEATTACH", "FIGATTACH", NULL );
	if ( lstatus!=SLIST_OK ) {
		*status = BIBL_ERR_MEMERR;
		return;
	}

	*status = urls_merge_and_add( in, LEVEL_ANY, out, "%U", LEVEL_MAIN, &types );

	slist_free( &types );
}

static void
append_trailer( fields *out, int *status )
{
	int fstatus;

	fstatus = fields_add( out, "%W", "PHY", LEVEL_MAIN );
	if ( fstatus!=FIELDS_OK ) {
		*status = BIBL_ERR_MEMERR;
		return;
	}

	fstatus = fields_add( out, "%G", "AUTHOR", LEVEL_MAIN );
	if ( fstatus!=FIELDS_OK ) {
		*status = BIBL_ERR_MEMERR;
		return;
	}
}

static int
adsout_assemble( fields *in, fields *out, param *pm, unsigned long refnum )
{
	int type, status = BIBL_OK;

	fields_clearused( in );
	type = get_type( in );

	append_Rtag   ( in, "%R", type, out, &status );
	append_people ( in, "AUTHOR", "AUTHOR:ASIS", "AUTHOR:CORP", "%A", LEVEL_MAIN, out, &status );
	append_people ( in, "EDITOR", "EDITOR:ASIS", "EDITOR:CORP", "%E", LEVEL_ANY,  out, &status );
	append_easy   ( in, "TITLE",	"%T", LEVEL_ANY, out, &status );
	append_titles ( in, type, out, &status );
	append_date   ( in,               "%D", LEVEL_ANY, out, &status );
	append_easy   ( in, "VOLUME",     "%V", LEVEL_ANY, out, &status );
	append_easy   ( in, "ISSUE",      "%N", LEVEL_ANY, out, &status );
	append_easy   ( in, "NUMBER",     "%N", LEVEL_ANY, out, &status );
	append_easy   ( in, "LANGUAGE",   "%M", LEVEL_ANY, out, &status );
	append_easyall( in, "NOTES",      "%X", LEVEL_ANY, out, NULL, &status );
	append_easy   ( in, "ABSTRACT",   "%B", LEVEL_ANY, out, &status );
	append_keys   ( in, "KEYWORD",    "%K", LEVEL_ANY, out, &status );
	append_urls   ( in, out, &status );
	append_pages  ( in, out, &status );
	append_easyall( in, "DOI",        "%Y", LEVEL_ANY, out, "DOI:", &status );
	append_trailer( out, &status );

	return status;
}

/*****************************************************
 PUBLIC: int adsout_write()
*****************************************************/

static int
adsout_write( fields *out, FILE *fp, param *p, unsigned long refnum )
{
	const char *tag, *value;
	int i;

	for ( i=0; i<out->n; ++i ) {
		tag   = fields_tag( out, i, FIELDS_CHRP );
		value = fields_value( out, i, FIELDS_CHRP );
		fprintf( fp, "%s %s\n", tag, value );
	}

	fprintf( fp, "\n" );
	fflush( fp );
	return BIBL_OK;
}
