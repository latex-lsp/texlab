/*
 * modsout.c
 *
 * Copyright (c) Chris Putnam 2003-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include "is_ws.h"
#include "str.h"
#include "charsets.h"
#include "str_conv.h"
#include "fields.h"
#include "iso639_2.h"
#include "utf8.h"
#include "modstypes.h"
#include "bu_auth.h"
#include "marc_auth.h"
#include "bibformats.h"

/*****************************************************
 PUBLIC: int modsout_initparams()
*****************************************************/

static void modsout_writeheader( FILE *outptr, param *p );
static void modsout_writefooter( FILE *outptr );
static int  modsout_write( fields *info, FILE *outptr, param *p, unsigned long numrefs );

int
modsout_initparams( param *pm, const char *progname )
{
	pm->writeformat      = BIBL_MODSOUT;
	pm->format_opts      = 0;
	pm->charsetout       = BIBL_CHARSET_UNICODE;
	pm->charsetout_src   = BIBL_SRC_DEFAULT;
	pm->latexout         = 0;
	pm->utf8out          = 1;
	pm->utf8bom          = 1;
	pm->xmlout           = BIBL_XMLOUT_TRUE;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->singlerefperfile = 0;

	pm->headerf   = modsout_writeheader;
	pm->footerf   = modsout_writefooter;
	pm->assemblef = NULL;
	pm->writef    = modsout_write;

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
 PUBLIC: int modsout_write()
*****************************************************/

/* output_tag()
 *
 * mode = TAG_OPEN,         "<tag>"
 * mode = TAG_CLOSE,        "</tag>"
 * mode = TAG_OPENCLOSE,    "<tag>data</tag>"
 * mode = TAG_SELFCLOSE,    "<tag/>"
 *
 * newline = TAG_NONEWLINE, "<tag>"
 * newline = TAG_NEWLINE,   "<tag>\n"
 *
 */
#define TAG_OPEN      (0)
#define TAG_CLOSE     (1)
#define TAG_OPENCLOSE (2)
#define TAG_SELFCLOSE (3)

#define TAG_NONEWLINE (0)
#define TAG_NEWLINE   (1)

static void
output_tag_core( FILE *outptr, int nindents, char *tag, char *data, unsigned char mode, unsigned char newline, va_list *attrs )
{
	char *attr, *val;
	int i;

	for ( i=0; i<nindents; ++i ) fprintf( outptr, "    " );

	if ( mode!=TAG_CLOSE )
		fprintf( outptr, "<" );
	else
		fprintf( outptr, "</" );

	fprintf( outptr, "%s", tag );

	do {
		attr = va_arg( *attrs, char * );
		if ( attr ) val  = va_arg( *attrs, char * );
		if ( attr && val )
			fprintf( outptr, " %s=\"%s\"", attr, val );
	} while ( attr && val );

	if ( mode!=TAG_SELFCLOSE )
		fprintf( outptr, ">" );
	else
		fprintf( outptr, "/>" );

	if ( mode==TAG_OPENCLOSE ) {
		fprintf( outptr, "%s</%s>", data, tag );
	}

	if ( newline==TAG_NEWLINE )
		fprintf( outptr, "\n" );
}

/* output_tag()
 *
 *     output XML tag
 *
 * mode     = [ TAG_OPEN | TAG_CLOSE | TAG_OPENCLOSE | TAG_SELFCLOSE ]
 * newline  = [ TAG_NEWLINE | TAG_NONEWLINE ]
 *
 * for mode TAG_OPENCLOSE, ensure that value is non-NULL, as string pointed to by value
 * will be output in the tag
 */
static void
output_tag( FILE *outptr, int nindents, char *tag, char *value, unsigned char mode, unsigned char newline, ... )
{
	va_list attrs;

	va_start( attrs, newline );
	output_tag_core( outptr, nindents, tag, value, mode, newline, &attrs );
	va_end( attrs );
}

/* output_fil()
 *
 *     output XML tag, but lookup data in fields struct
 *
 * mode     = [ TAG_OPEN | TAG_CLOSE | TAG_OPENCLOSE | TAG_SELFCLOSE ]
 * newline  = [ TAG_NEWLINE | TAG_NONEWLINE ]
 *
 * value looked up in fields will only be used in mode TAG_OPENCLOSE
 */
static void
output_fil( FILE *outptr, int nindents, char *tag, fields *f, int n, unsigned char mode, unsigned char newline, ... )
{
	va_list attrs;
	char *value;

	if ( n!=-1 ) {
		value = (char *) fields_value( f, n, FIELDS_CHRP );
		va_start( attrs, newline );
		output_tag_core( outptr, nindents, tag, value, mode, newline, &attrs );
		va_end( attrs );
	}
}

static inline int
lvl2indent( int level )
{
	if ( level < -1 ) return -level + 1;
	else return level + 1;
}

static inline int
incr_level( int level, int amt )
{
	if ( level > -1 ) return level+amt;
	else return level-amt;
}

/* convert_findallfields()
 *
 *       Find the positions of all convert.internal tags in the fields
 *       structure and store the locations in convert.pos element.
 *
 *       Return number of the tags found.
 */
static int
convert_findallfields( fields *f, convert *parts, int nparts, int level )
{
	int i, n = 0;

	for ( i=0; i<nparts; ++i ) {
		parts[i].pos = fields_find( f, parts[i].internal, level );
		n += ( parts[i].pos!=FIELDS_NOTFOUND );
	}

	return n;
}

static void
output_title( fields *f, FILE *outptr, int level )
{
	int ttl    = fields_find( f, "TITLE", level );
	int subttl = fields_find( f, "SUBTITLE", level );
	int shrttl = fields_find( f, "SHORTTITLE", level );
	int parttl = fields_find( f, "PARTTITLE", level );
	char *val;

	output_tag( outptr, lvl2indent(level),               "titleInfo", NULL,      TAG_OPEN,      TAG_NEWLINE, NULL );
	output_fil( outptr, lvl2indent(incr_level(level,1)), "title",     f, ttl,    TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	output_fil( outptr, lvl2indent(incr_level(level,1)), "subTitle",  f, subttl, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	output_fil( outptr, lvl2indent(incr_level(level,1)), "partName",  f, parttl, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	/* MODS output doesn't verify if we don't at least have a <title/> element */
	if ( ttl==-1 && subttl==-1 )
		output_tag( outptr, lvl2indent(incr_level(level,1)), "title", NULL,  TAG_SELFCLOSE, TAG_NEWLINE, NULL );
	output_tag( outptr, lvl2indent(level),               "titleInfo", NULL,      TAG_CLOSE,     TAG_NEWLINE, NULL );

	/* output shorttitle if it's different from normal title */
	if ( shrttl!=FIELDS_NOTFOUND ) {
		val = (char *) fields_value( f, shrttl, FIELDS_CHRP );
		if ( ttl==FIELDS_NOTFOUND || subttl!=FIELDS_NOTFOUND || strcmp(f->data[ttl].data,val) ) {
			output_tag( outptr, lvl2indent(level),               "titleInfo", NULL, TAG_OPEN,      TAG_NEWLINE, "type", "abbreviated", NULL );
			output_tag( outptr, lvl2indent(incr_level(level,1)), "title",     val,  TAG_OPENCLOSE, TAG_NEWLINE, NULL );
			output_tag( outptr, lvl2indent(level),               "titleInfo", NULL, TAG_CLOSE,     TAG_NEWLINE, NULL );
		}
	}
}

static void
output_name( FILE *outptr, char *p, int level )
{
	str family, part, suffix;
	int n=0;

	strs_init( &family, &part, &suffix, NULL );

	while ( *p && *p!='|' ) str_addchar( &family, *p++ );
	if ( *p=='|' ) p++;

	while ( *p ) {
		while ( *p && *p!='|' ) str_addchar( &part, *p++ );
		/* truncate periods from "A. B. Jones" names */
		if ( part.len ) {
			if ( part.len==2 && part.data[1]=='.' ) {
				part.len=1;
				part.data[1]='\0';
			}
			if ( n==0 )
				output_tag( outptr, lvl2indent(level), "name", NULL, TAG_OPEN, TAG_NEWLINE, "type", "personal", NULL );
			output_tag( outptr, lvl2indent(incr_level(level,1)), "namePart", part.data, TAG_OPENCLOSE, TAG_NEWLINE, "type", "given", NULL );
			n++;
		}
		if ( *p=='|' ) {
			p++;
			if ( *p=='|' ) {
				p++;
				while ( *p && *p!='|' ) str_addchar( &suffix, *p++ );
			}
			str_empty( &part );
		}
	}

	if ( family.len ) {
		if ( n==0 )
			output_tag( outptr, lvl2indent(level), "name", NULL, TAG_OPEN, TAG_NEWLINE, "type", "personal", NULL );
		output_tag( outptr, lvl2indent(incr_level(level,1)), "namePart", family.data, TAG_OPENCLOSE, TAG_NEWLINE, "type", "family", NULL );
		n++;
	}

	if ( suffix.len ) {
		if ( n==0 )
			output_tag( outptr, lvl2indent(level), "name", NULL, TAG_OPEN, TAG_NEWLINE, "type", "personal", NULL );
		output_tag( outptr, lvl2indent(incr_level(level,1)), "namePart", suffix.data, TAG_OPENCLOSE, TAG_NEWLINE, "type", "suffix", NULL );
	}

	strs_free( &part, &family, &suffix, NULL );
}


/* MODS v 3.4
 *
 * <name [type="corporation"/type="conference"]>
 *    <namePart></namePart>
 *    <displayForm></displayForm>
 *    <affiliation></affiliation>
 *    <role>
 *        <roleTerm [authority="marcrealtor"] type="text"></roleTerm>
 *    </role>
 *    <description></description>
 * </name>
 */

#define NO_AUTHORITY (0)
#define MARC_AUTHORITY (1)

static void
output_names( fields *f, FILE *outptr, int level )
{
	convert   names[] = {
	  { "author",                              "AUTHOR",          0, MARC_AUTHORITY },
	  { "editor",                              "EDITOR",          0, MARC_AUTHORITY },
	  { "annotator",                           "ANNOTATOR",       0, MARC_AUTHORITY },
	  { "artist",                              "ARTIST",          0, MARC_AUTHORITY },
	  { "author",                              "2ND_AUTHOR",      0, MARC_AUTHORITY },
	  { "author",                              "3RD_AUTHOR",      0, MARC_AUTHORITY },
	  { "author",                              "SUB_AUTHOR",      0, MARC_AUTHORITY },
	  { "author",                              "COMMITTEE",       0, MARC_AUTHORITY },
	  { "author",                              "COURT",           0, MARC_AUTHORITY },
	  { "author",                              "LEGISLATIVEBODY", 0, MARC_AUTHORITY },
	  { "author of afterword, colophon, etc.", "AFTERAUTHOR",     0, MARC_AUTHORITY },
	  { "author of introduction, etc.",        "INTROAUTHOR",     0, MARC_AUTHORITY },
	  { "cartographer",                        "CARTOGRAPHER",    0, MARC_AUTHORITY },
	  { "collaborator",                        "COLLABORATOR",    0, MARC_AUTHORITY },
	  { "commentator",                         "COMMENTATOR",     0, MARC_AUTHORITY },
	  { "compiler",                            "COMPILER",        0, MARC_AUTHORITY },
	  { "degree grantor",                      "DEGREEGRANTOR",   0, MARC_AUTHORITY },
	  { "director",                            "DIRECTOR",        0, MARC_AUTHORITY },
	  { "event",                               "EVENT",           0, NO_AUTHORITY   },
	  { "inventor",                            "INVENTOR",        0, MARC_AUTHORITY },
	  { "organizer of meeting",                "ORGANIZER",       0, MARC_AUTHORITY },
	  { "patent holder",                       "ASSIGNEE",        0, MARC_AUTHORITY },
	  { "performer",                           "PERFORMER",       0, MARC_AUTHORITY },
	  { "producer",                            "PRODUCER",        0, MARC_AUTHORITY },
	  { "addressee",                           "ADDRESSEE",       0, MARC_AUTHORITY },
	  { "redactor",                            "REDACTOR",        0, MARC_AUTHORITY },
	  { "reporter",                            "REPORTER",        0, MARC_AUTHORITY },
	  { "sponsor",                             "SPONSOR",         0, MARC_AUTHORITY },
	  { "translator",                          "TRANSLATOR",      0, MARC_AUTHORITY },
	  { "writer",                              "WRITER",          0, MARC_AUTHORITY },
	};
	int i, n, nfields, ntypes = sizeof( names ) / sizeof( convert );
	int f_asis, f_corp, f_conf;
	str role;

	str_init( &role );
	nfields = fields_num( f );
	for ( n=0; n<ntypes; ++n ) {
		for ( i=0; i<nfields; ++i ) {
			if ( fields_level( f, i )!=level ) continue;
			if ( f->data[i].len==0 ) continue;
			f_asis = f_corp = f_conf = 0;
			str_strcpyc( &role, f->tag[i].data );
			if ( str_findreplace( &role, ":ASIS", "" )) f_asis=1;
			if ( str_findreplace( &role, ":CORP", "" )) f_corp=1;
			if ( str_findreplace( &role, ":CONF", "" )) f_conf=1;
			if ( strcasecmp( role.data, names[n].internal ) )
				continue;
			if ( f_asis ) {
				output_tag( outptr, lvl2indent(level),               "name",     NULL, TAG_OPEN,      TAG_NEWLINE, NULL );
				output_fil( outptr, lvl2indent(incr_level(level,1)), "namePart", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
			} else if ( f_corp ) {
				output_tag( outptr, lvl2indent(level),               "name",     NULL, TAG_OPEN,      TAG_NEWLINE, "type", "corporate", NULL );
				output_fil( outptr, lvl2indent(incr_level(level,1)), "namePart", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
			} else if ( f_conf ) {
				output_tag( outptr, lvl2indent(level),               "name",     NULL, TAG_OPEN,      TAG_NEWLINE, "type", "conference", NULL );
				output_fil( outptr, lvl2indent(incr_level(level,1)), "namePart", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
			} else {
				output_name(outptr, f->data[i].data, level);
			}
			output_tag( outptr, lvl2indent(incr_level(level,1)), "role", NULL, TAG_OPEN, TAG_NEWLINE, NULL );
			if ( names[n].code & MARC_AUTHORITY )
				output_tag( outptr, lvl2indent(incr_level(level,2)), "roleTerm", names[n].mods, TAG_OPENCLOSE, TAG_NEWLINE, "authority", "marcrelator", "type", "text", NULL );
			else
				output_tag( outptr, lvl2indent(incr_level(level,2)), "roleTerm", names[n].mods, TAG_OPENCLOSE, TAG_NEWLINE, "type", "text", NULL );
			output_tag( outptr, lvl2indent(incr_level(level,1)), "role", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
			output_tag( outptr, lvl2indent(level),               "name", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
			fields_setused( f, i );
		}
	}
	str_free( &role );
}

/* datepos[ NUM_DATE_TYPES ]
 *     use define to ensure that the array and loops don't get out of sync
 *     datepos[0] -> DATE:YEAR/PARTDATE:YEAR
 *     datepos[1] -> DATE:MONTH/PARTDATE:MONTH
 *     datepos[2] -> DATE:DAY/PARTDATE:DAY
 *     datepos[3] -> DATE/PARTDATE
 */
#define DATE_YEAR      (0)
#define DATE_MONTH     (1)
#define DATE_DAY       (2)
#define DATE_ALL       (3)
#define NUM_DATE_TYPES (4)

static int
find_datepos( fields *f, int level, unsigned char use_altnames, int datepos[NUM_DATE_TYPES] )
{
	char      *src_names[] = { "DATE:YEAR", "DATE:MONTH", "DATE:DAY", "DATE" };
	char      *alt_names[] = { "PARTDATE:YEAR", "PARTDATE:MONTH", "PARTDATE:DAY", "PARTDATE" };
	int       found = 0;
	int       i;

	for ( i=0; i<NUM_DATE_TYPES; ++i ) {
		if ( !use_altnames )
			datepos[i] = fields_find( f, src_names[i], level );
		else
			datepos[i] = fields_find( f, alt_names[i], level );
		if ( datepos[i]!=FIELDS_NOTFOUND ) found = 1;
	}

	return found;
}

/* find_dateinfo()
 *
 *      fill datepos[] array with position indexes to date information in fields *f
 *
 *      when generating dates for LEVEL_MAIN, first look at level=LEVEL_MAIN, but if that
 *      fails, use LEVEL_ANY (-1)
 *
 *      returns 1 if date information found, 0 otherwise
 */
static int
find_dateinfo( fields *f, int level, int datepos[ NUM_DATE_TYPES ] )
{
	int found;

	/* default to finding date information for the current level */
	found = find_datepos( f, level, 0, datepos );

	/* for LEVEL_MAIN, do whatever it takes to find a date */
	if ( !found && level == LEVEL_MAIN ) {
		found = find_datepos( f, -1, 0, datepos );
	}
	if ( !found && level == LEVEL_MAIN ) {
		found = find_datepos( f, -1, 1, datepos );
	}

	return found;
}

static void
output_datepieces( fields *f, FILE *outptr, int pos[ NUM_DATE_TYPES ] )
{
	str *s;
	int i;

	for ( i=0; i<3 && pos[i]!=-1; ++i ) {
		if ( i>0 ) fprintf( outptr, "-" );
		/* zero pad month or days written as "1", "2", "3" ... */
		if ( i==DATE_MONTH || i==DATE_DAY ) {
			s = fields_value( f, pos[i], FIELDS_STRP_NOUSE );
			if ( s->len==1 ) {
				fprintf( outptr, "0" );
			}
		}
		fprintf( outptr, "%s", (char *) fields_value( f, pos[i], FIELDS_CHRP ) );
	}
}

static void
output_dateissued( fields *f, FILE *outptr, int level, int pos[ NUM_DATE_TYPES ] )
{
	output_tag( outptr, lvl2indent(incr_level(level,1)), "dateIssued", NULL, TAG_OPEN, TAG_NONEWLINE, NULL );
	if ( pos[ DATE_YEAR ]!=-1 || pos[ DATE_MONTH ]!=-1 || pos[ DATE_DAY ]!=-1 ) {
		output_datepieces( f, outptr, pos );
	} else {
		fprintf( outptr, "%s", (char *) fields_value( f, pos[ DATE_ALL ], FIELDS_CHRP ) );
	}
	fprintf( outptr, "</dateIssued>\n" );
}

static void
output_origin( fields *f, FILE *outptr, int level )
{
	convert parts[] = {
		{ "issuance",	  "ISSUANCE",          0, 0 },
		{ "publisher",	  "PUBLISHER",         0, 0 },
		{ "place",	  "ADDRESS",           0, 1 },
		{ "place",        "ADDRESS:PUBLISHER", 0, 0 },
		{ "place",	  "ADDRESS:AUTHOR",    0, 0 },
		{ "edition",	  "EDITION",           0, 0 },
		{ "dateCaptured", "URLDATE",           0, 0 }
	};
	int nparts = sizeof( parts ) / sizeof( parts[0] );
	int i, found, datefound, datepos[ NUM_DATE_TYPES ];

	found     = convert_findallfields( f, parts, nparts, level );
	datefound = find_dateinfo( f, level, datepos );
	if ( !found && !datefound ) return;


	output_tag( outptr, lvl2indent(level), "originInfo", NULL, TAG_OPEN, TAG_NEWLINE, NULL );

	/* issuance must precede date */
	if ( parts[0].pos!=-1 )
		output_fil( outptr, lvl2indent(incr_level(level,1)), "issuance", f, parts[0].pos, TAG_OPENCLOSE, TAG_NEWLINE, NULL );

	/* date */
	if ( datefound )
		output_dateissued( f, outptr, level, datepos );

	/* rest of the originInfo elements */
	for ( i=1; i<nparts; i++ ) {

		/* skip missing originInfo elements */
		if ( parts[i].pos==-1 ) continue;

		/* normal originInfo element */
		if ( parts[i].code==0 ) {
			output_fil( outptr, lvl2indent(incr_level(level,1)), parts[i].mods, f, parts[i].pos, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
		}

		/* originInfo with placeTerm info */
		else {
			output_tag( outptr, lvl2indent(incr_level(level,1)), parts[i].mods, NULL,            TAG_OPEN,      TAG_NEWLINE, NULL );
			output_fil( outptr, lvl2indent(incr_level(level,2)), "placeTerm",   f, parts[i].pos, TAG_OPENCLOSE, TAG_NEWLINE, "type", "text", NULL );
			output_tag( outptr, lvl2indent(incr_level(level,1)), parts[i].mods, NULL,            TAG_CLOSE,     TAG_NEWLINE, NULL );
		}
	}

	output_tag( outptr, lvl2indent(level), "originInfo", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
}

/* output_language_core()
 *
 *      generates language output for tag="langauge" or tag="languageOfCataloging"
 *      if possible, outputs iso639-2b code for the language
 *
 * <language>
 *     <languageTerm type="text">xxx</languageTerm>
 * </language>
 *
 * <language>
 *     <languageTerm type="text">xxx</languageTerm>
 *     <languageTerm type="code" authority="iso639-2b">xxx</languageTerm>
 * </language>
 *
 */
static void
output_language_core( fields *f, int n, FILE *outptr, char *tag, int level )
{
	char *lang, *code;

	lang = (char *) fields_value( f, n, FIELDS_CHRP );
	code = iso639_2_from_language( lang );

	output_tag( outptr, lvl2indent(level),               tag,            NULL, TAG_OPEN,      TAG_NEWLINE, NULL );
	output_tag( outptr, lvl2indent(incr_level(level,1)), "languageTerm", lang, TAG_OPENCLOSE, TAG_NEWLINE, "type", "text", NULL );
	if ( code ) {
		output_tag( outptr, lvl2indent(incr_level(level,1)), "languageTerm", code, TAG_OPENCLOSE, TAG_NEWLINE, "type", "code", "authority", "iso639-2b", NULL );
	}
	output_tag( outptr, lvl2indent(level),               tag,            NULL, TAG_CLOSE,     TAG_NEWLINE, NULL );
}

static void
output_language( fields *f, FILE *outptr, int level )
{
	int n;
	n = fields_find( f, "LANGUAGE", level );
	if ( n!=FIELDS_NOTFOUND )
		output_language_core( f, n, outptr, "language", level );
}

static void
output_description( fields *f, FILE *outptr, int level )
{
	char *val;
	int n;

	n = fields_find( f, "DESCRIPTION", level );
	if ( n!=FIELDS_NOTFOUND ) {
		val = ( char * ) fields_value( f, n, FIELDS_CHRP );
		output_tag( outptr, lvl2indent(level),               "physicalDescription", NULL, TAG_OPEN,      TAG_NEWLINE, NULL );
		output_tag( outptr, lvl2indent(incr_level(level,1)), "note",                val,  TAG_OPENCLOSE, TAG_NEWLINE, NULL );
		output_tag( outptr, lvl2indent(level),               "physicalDescription", NULL, TAG_CLOSE,     TAG_NEWLINE, NULL );
	}
}

static void
output_toc( fields *f, FILE *outptr, int level )
{
	char *val;
	int n;

	n = fields_find( f, "CONTENTS", level );
	if ( n!=FIELDS_NOTFOUND ) {
		val = (char *) fields_value( f, n, FIELDS_CHRP );
		output_tag( outptr, lvl2indent(level), "tableOfContents", val, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	}
}

/* detail output
 *
 * for example:
 *
 * <detail type="volume"><number>xxx</number></detail
 */
static void
mods_output_detail( fields *f, FILE *outptr, int n, char *item_name, int level )
{
	if ( n!=-1 ) {
		output_tag( outptr, lvl2indent(incr_level(level,1)), "detail", NULL,  TAG_OPEN,      TAG_NONEWLINE, "type", item_name, NULL );
		output_fil( outptr, 0,                                "number", f, n,  TAG_OPENCLOSE, TAG_NONEWLINE, NULL );
		output_tag( outptr, 0,                                "detail", NULL,  TAG_CLOSE,     TAG_NEWLINE,   NULL );                       
	}
}


/* extents output
 *
 * <extent unit="page">
 * 	<start>xxx</start>
 * 	<end>xxx</end>
 * </extent>
 */
static void
mods_output_extents( fields *f, FILE *outptr, int start, int end, int total, char *extype, int level )
{
	char *val;

	output_tag( outptr, lvl2indent(incr_level(level,1)), "extent", NULL, TAG_OPEN, TAG_NEWLINE, "unit", extype, NULL );
	if ( start!=-1 ) {
		val = (char *) fields_value( f, start, FIELDS_CHRP );
		output_tag( outptr, lvl2indent(incr_level(level,2)), "start", val, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	}
	if ( end!=-1 ) {
		val = (char *) fields_value( f, end, FIELDS_CHRP );
		output_tag( outptr, lvl2indent(incr_level(level,2)), "end",   val, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	}
	if ( total!=-1 ) {
		val = (char *) fields_value( f, total, FIELDS_CHRP );
		output_tag( outptr, lvl2indent(incr_level(level,2)), "total", val, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	}
	output_tag( outptr, lvl2indent(incr_level(level,1)), "extent", NULL, TAG_CLOSE,     TAG_NEWLINE, NULL );
}

static void
try_output_partheader( FILE *outptr, int wrote_header, int level )
{
	if ( !wrote_header )
		output_tag( outptr, lvl2indent(level), "part", NULL, TAG_OPEN, TAG_NEWLINE, NULL );
}

static void
try_output_partfooter( FILE *outptr, int wrote_header, int level )
{
	if ( wrote_header )
		output_tag( outptr, lvl2indent(level), "part", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
}

/* part date output
 *
 * <date>xxxx-xx-xx</date>
 *
 */
static int
output_partdate( fields *f, FILE *outptr, int level, int wrote_header )
{
	convert parts[] = {
		{ "",	"PARTDATE:YEAR",           0, 0 },
		{ "",	"PARTDATE:MONTH",          0, 0 },
		{ "",	"PARTDATE:DAY",            0, 0 },
	};
	int nparts = sizeof(parts)/sizeof(parts[0]);

	if ( !convert_findallfields( f, parts, nparts, level ) ) return 0;

	try_output_partheader( outptr, wrote_header, level );

	output_tag( outptr, lvl2indent(incr_level(level,1)), "date", NULL, TAG_OPEN, TAG_NONEWLINE, NULL );

	if ( parts[0].pos!=-1 ) {
		fprintf( outptr, "%s", (char *) fields_value( f, parts[0].pos, FIELDS_CHRP ) );
	} else fprintf( outptr, "XXXX" );

	if ( parts[1].pos!=-1 ) {
		fprintf( outptr, "-%s", (char *) fields_value( f, parts[1].pos, FIELDS_CHRP ) );
	}

	if ( parts[2].pos!=-1 ) {
		if ( parts[1].pos==-1 )
			fprintf( outptr, "-XX" );
		fprintf( outptr, "-%s", (char *) fields_value( f, parts[2].pos, FIELDS_CHRP ) );
	}

	fprintf( outptr,"</date>\n");

	return 1;
}

static int
output_partpages( fields *f, FILE *outptr, int level, int wrote_header )
{
	convert parts[] = {
		{ "",  "PAGES:START",              0, 0 },
		{ "",  "PAGES:STOP",               0, 0 },
		{ "",  "PAGES",                    0, 0 },
		{ "",  "PAGES:TOTAL",              0, 0 }
	};
	int nparts = sizeof(parts)/sizeof(parts[0]);

	if ( !convert_findallfields( f, parts, nparts, level ) ) return 0;

	try_output_partheader( outptr, wrote_header, level );

	/* If PAGES:START or PAGES:STOP are undefined */
	if ( parts[0].pos==-1 || parts[1].pos==-1 ) {
		if ( parts[0].pos!=-1 )
			mods_output_detail( f, outptr, parts[0].pos, "page", level );
		if ( parts[1].pos!=-1 )
			mods_output_detail( f, outptr, parts[1].pos, "page", level );
		if ( parts[2].pos!=-1 )
			mods_output_detail( f, outptr, parts[2].pos, "page", level );
		if ( parts[3].pos!=-1 )
			mods_output_extents( f, outptr, -1, -1, parts[3].pos, "page", level );
	}
	/* If both PAGES:START and PAGES:STOP are defined */
	else {
		mods_output_extents( f, outptr, parts[0].pos, parts[1].pos, parts[3].pos, "page", level );
	}

	return 1;
}

static int
output_partelement( fields *f, FILE *outptr, int level, int wrote_header )
{
	convert parts[] = {
		{ "",                "NUMVOLUMES",      0, 0 },
		{ "volume",          "VOLUME",          0, 0 },
		{ "section",         "SECTION",         0, 0 },
		{ "issue",           "ISSUE",           0, 0 },
		{ "number",          "NUMBER",          0, 0 },
		{ "publiclawnumber", "PUBLICLAWNUMBER", 0, 0 },
		{ "session",         "SESSION",         0, 0 },
		{ "articlenumber",   "ARTICLENUMBER",   0, 0 },
		{ "part",            "PART",            0, 0 },
		{ "chapter",         "CHAPTER",         0, 0 },
		{ "report number",   "REPORTNUMBER",    0, 0 },
	};
	int i, nparts = sizeof( parts ) / sizeof( convert );

	if ( !convert_findallfields( f, parts, nparts, level ) ) return 0;

	try_output_partheader( outptr, wrote_header, level );

	/* start loop at 1 to skip NUMVOLUMES */
	for ( i=1; i<nparts; ++i ) {
		if ( parts[i].pos==-1 ) continue;
		mods_output_detail( f, outptr, parts[i].pos, parts[i].mods, level );
	}

	if ( parts[0].pos!=-1 )
		mods_output_extents( f, outptr, -1, -1, parts[0].pos, "volumes", level );

	return 1;
}

static void
output_part( fields *f, FILE *outptr, int level )
{
	int wrote_hdr;
	wrote_hdr  = output_partdate( f, outptr, level, 0 );
	wrote_hdr += output_partelement( f, outptr, level, wrote_hdr );
	wrote_hdr += output_partpages( f, outptr, level, wrote_hdr );
	try_output_partfooter( outptr, wrote_hdr, level );
}

static void
output_recordInfo( fields *f, FILE *outptr, int level )
{
	int n;
	n = fields_find( f, "LANGCATALOG", level );
	if ( n!=FIELDS_NOTFOUND ) {
		output_tag( outptr, lvl2indent(level), "recordInfo", NULL, TAG_OPEN, TAG_NEWLINE, NULL );
		output_language_core( f, n, outptr, "languageOfCataloging", incr_level(level,1) );
		output_tag( outptr, lvl2indent(level), "recordInfo", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
	}
}

/* output_genre()
 *
 * <genre authority="marcgt">thesis</genre>
 * <genre authority="bibutilsgt">Diploma thesis</genre>
 */
static void
output_genre( fields *f, FILE *outptr, int level )
{
	char *value, *attr = NULL, *attrvalue = NULL;
	int i, n;

	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( fields_level( f, i ) != level ) continue;
		if ( !fields_match_tag( f, i, "GENRE:MARC" ) && !fields_match_tag( f, i, "GENRE:BIBUTILS" ) && !fields_match_tag( f, i, "GENRE:UNKNOWN" ) ) continue;
		value = fields_value( f, i, FIELDS_CHRP );
		if ( is_marc_genre( value ) ) {
			attr      = "authority";
			attrvalue = "marcgt";
		}
		else if ( is_bu_genre( value ) ) {
			attr      = "authority";
			attrvalue = "bibutilsgt";
		}
		output_tag( outptr, lvl2indent(level), "genre", value, TAG_OPENCLOSE, TAG_NEWLINE, attr, attrvalue, NULL );
	}
}

/* output_resource()
 *
 * <typeOfResource>text</typeOfResource>
 */
static void
output_resource( fields *f, FILE *outptr, int level )
{
	char *value;
	int n;

	n = fields_find( f, "RESOURCE", level );
	if ( n!=FIELDS_NOTFOUND ) {
		value = fields_value( f, n, FIELDS_CHRP );
		if ( is_marc_resource( value ) ) {
			output_fil( outptr, lvl2indent(level), "typeOfResource", f, n, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
		} else {
			fprintf( stderr, "Illegal typeofResource = '%s'\n", value );
		}
	}
}

static void
output_type( fields *f, FILE *outptr, int level )
{
	int n;

	/* silence warnings about INTERNAL_TYPE being unused */
	n = fields_find( f, "INTERNAL_TYPE", LEVEL_MAIN );
	if ( n!=FIELDS_NOTFOUND ) fields_setused( f, n );

	output_resource( f, outptr, level );
	output_genre( f, outptr, level );
}

/* output_abs()
 *
 * <abstract>xxxx</abstract>
 */
static void
output_abs( fields *f, FILE *outptr, int level )
{
	int n;

	n = fields_find( f, "ABSTRACT", level );
	output_fil( outptr, lvl2indent(level), "abstract", f, n, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
}

static void
output_notes( fields *f, FILE *outptr, int level )
{
	int i, n;
	char *t;

	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( fields_level( f, i ) != level ) continue;
		t = fields_tag( f, i, FIELDS_CHRP_NOUSE );
		if ( !strcasecmp( t, "NOTES" ) )
			output_fil( outptr, lvl2indent(level), "note", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
		else if ( !strcasecmp( t, "PUBSTATE" ) )
			output_fil( outptr, lvl2indent(level), "note", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "type", "publication status", NULL );
		else if ( !strcasecmp( t, "ANNOTE" ) )
			output_fil( outptr, lvl2indent(level), "bibtex-annote", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
		else if ( !strcasecmp( t, "TIMESCITED" ) )
			output_fil( outptr, lvl2indent(level), "note", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "type", "times cited", NULL );
		else if ( !strcasecmp( t, "ANNOTATION" ) )
			output_fil( outptr, lvl2indent(level), "note", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "type", "annotation", NULL );
		else if ( !strcasecmp( t, "ADDENDUM" ) )
			output_fil( outptr, lvl2indent(level), "note", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "type", "addendum", NULL );
		else if ( !strcasecmp( t, "BIBKEY" ) )
			output_fil( outptr, lvl2indent(level), "note", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "type", "bibliography key", NULL );
	}
}

/* output_key()
 *
 * <subject>
 *    <topic>xxxx</topic>
 * </subject>
 */
static void
output_key( fields *f, FILE *outptr, int level )
{
	int i, n;

	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( fields_level( f, i ) != level ) continue;
		if ( !strcasecmp( f->tag[i].data, "KEYWORD" ) ) {
			output_tag( outptr, lvl2indent(level),               "subject", NULL, TAG_OPEN,      TAG_NEWLINE, NULL );
			output_fil( outptr, lvl2indent(incr_level(level,1)), "topic",   f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
			output_tag( outptr, lvl2indent(level),               "subject", NULL, TAG_CLOSE,     TAG_NEWLINE, NULL );
		}
		else if ( !strcasecmp( f->tag[i].data, "EPRINTCLASS" ) ) {
			output_tag( outptr, lvl2indent(level),               "subject", NULL, TAG_OPEN,      TAG_NEWLINE, NULL );
			output_fil( outptr, lvl2indent(incr_level(level,1)), "topic",   f, i, TAG_OPENCLOSE, TAG_NEWLINE, "class", "primary", NULL );
			output_tag( outptr, lvl2indent(level),               "subject", NULL, TAG_CLOSE,     TAG_NEWLINE, NULL );
		}
	}
}

static void
output_sn( fields *f, FILE *outptr, int level )
{
	convert sn_types[] = {
		{ "isbn",      "ISBN",      0, 0 },
		{ "isbn",      "ISBN13",    0, 0 },
		{ "lccn",      "LCCN",      0, 0 },
		{ "issn",      "ISSN",      0, 0 },
		{ "coden",     "CODEN",     0, 0 },
		{ "citekey",   "REFNUM",    0, 0 },
		{ "doi",       "DOI",       0, 0 },
		{ "eid",       "EID",       0, 0 },
		{ "eprint",    "EPRINT",    0, 0 },
		{ "eprinttype","EPRINTTYPE",0, 0 },
		{ "pubmed",    "PMID",      0, 0 },
		{ "MRnumber",  "MRNUMBER",  0, 0 },
		{ "medline",   "MEDLINE",   0, 0 },
		{ "pii",       "PII",       0, 0 },
		{ "pmc",       "PMC",       0, 0 },
		{ "arXiv",     "ARXIV",     0, 0 },
		{ "isi",       "ISIREFNUM", 0, 0 },
		{ "accessnum", "ACCESSNUM", 0, 0 },
		{ "jstor",     "JSTOR",     0, 0 },
		{ "isrn",      "ISRN",      0, 0 },
	};
	int ntypes = sizeof( sn_types ) / sizeof( sn_types[0] );
	int i, n, found;

	/* output call number */
	n = fields_find( f, "CALLNUMBER", level );
	output_fil( outptr, lvl2indent(level), "classification", f, n, TAG_OPENCLOSE, TAG_NEWLINE, NULL );

	/* output specialized serialnumber */
	found = convert_findallfields( f, sn_types, ntypes, level );
	if ( found ) {
		for ( i=0; i<ntypes; ++i ) {
			if ( sn_types[i].pos==-1 ) continue;
			output_fil( outptr, lvl2indent(level), "identifier", f, sn_types[i].pos, TAG_OPENCLOSE, TAG_NEWLINE, "type", sn_types[i].mods, NULL );
		}
	}

	/* output _all_ elements of type SERIALNUMBER */
	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( f->level[i]!=level ) continue;
		if ( strcasecmp( f->tag[i].data, "SERIALNUMBER" ) ) continue;
		output_fil( outptr, lvl2indent(level), "identifier", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "type", "serial number", NULL );
	}
}

/* output_url()
 *
 * <location>
 *     <url>URL</url>
 *     <url urlType="pdf">PDFLINK</url>
 *     <url displayLabel="Electronic full text" access="raw object">PDFLINK</url>
 *     <physicalLocation>LOCATION</physicalLocation>
 * </location>
 */
static void
output_url( fields *f, FILE *outptr, int level )
{
	int location   = fields_find( f, "LOCATION",   level );
	int url        = fields_find( f, "URL",        level );
	int fileattach = fields_find( f, "FILEATTACH", level );
	int pdflink    = fields_find( f, "PDFLINK",    level );
	int i, n;

	if ( url==FIELDS_NOTFOUND && location==FIELDS_NOTFOUND && pdflink==FIELDS_NOTFOUND && fileattach==FIELDS_NOTFOUND ) return;
	output_tag( outptr, lvl2indent(level), "location", NULL, TAG_OPEN, TAG_NEWLINE, NULL );

	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( f->level[i]!=level ) continue;
		if ( strcasecmp( f->tag[i].data, "URL" ) ) continue;
		output_fil( outptr, lvl2indent(incr_level(level,1)), "url", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	}
	for ( i=0; i<n; ++i ) {
		if ( f->level[i]!=level ) continue;
		if ( strcasecmp( f->tag[i].data, "PDFLINK" ) ) continue;
/*		output_fil( outptr, lvl2indent(incr_level(level,1)), "url", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "urlType", "pdf", NULL ); */
		output_fil( outptr, lvl2indent(incr_level(level,1)), "url", f, i, TAG_OPENCLOSE, TAG_NEWLINE, NULL );
	}
	for ( i=0; i<n; ++i ) {
		if ( f->level[i]!=level ) continue;
		if ( strcasecmp( f->tag[i].data, "FILEATTACH" ) ) continue;
		output_fil( outptr, lvl2indent(incr_level(level,1)), "url", f, i, TAG_OPENCLOSE, TAG_NEWLINE, "displayLabel", "Electronic full text", "access", "raw object", NULL );
	}
	if ( location!=-1 )
		output_fil( outptr, lvl2indent(incr_level(level,1)), "physicalLocation", f, location, TAG_OPENCLOSE, TAG_NEWLINE, NULL );

	output_tag( outptr, lvl2indent(level), "location", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
}

/* refnum should start with a non-number and not include spaces -- ignore this */
static void
output_refnum( fields *f, int n, FILE *outptr )
{
	char *p = fields_value( f, n, FIELDS_CHRP_NOUSE );
/*	if ( p && ((*p>='0' && *p<='9') || *p=='-' || *p=='_' ))
		fprintf( outptr, "ref" );*/
	while ( p && *p ) {
		if ( !is_ws(*p) ) fprintf( outptr, "%c", *p );
/*		if ( (*p>='A' && *p<='Z') ||
		     (*p>='a' && *p<='z') ||
		     (*p>='0' && *p<='9') ||
		     (*p=='-') || (*p=='
		     (*p=='_') ) fprintf( outptr, "%c", *p );*/
		p++;
	}
}

static void
output_head( fields *f, FILE *outptr, int dropkey, unsigned long numrefs )
{
	int n;
	fprintf( outptr, "<mods");
	if ( !dropkey ) {
		n = fields_find( f, "REFNUM", LEVEL_MAIN );
		if ( n!=FIELDS_NOTFOUND ) {
			fprintf( outptr, " ID=\"");
			output_refnum( f, n, outptr );
			fprintf( outptr, "\"");
		}
	}
	fprintf( outptr, ">\n" );
}

static int
original_items( fields *f, int level )
{
	int i, targetlevel, n;
	if ( level < 0 ) return 0;
	targetlevel = -( level + 2 );
	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( fields_level( f, i ) == targetlevel )
			return targetlevel;
	}
	return 0;
}

static void
output_citeparts( fields *f, FILE *outptr, int level, int max )
{
	int orig_level;

	output_title(       f, outptr, level );
	output_names(       f, outptr, level );
	output_origin(      f, outptr, level );
	output_type(        f, outptr, level );
	output_language(    f, outptr, level );
	output_description( f, outptr, level );

	if ( level >= 0 && level < max ) {
		output_tag( outptr, lvl2indent(level), "relatedItem", NULL, TAG_OPEN,  TAG_NEWLINE, "type", "host", NULL );
		output_citeparts( f, outptr, incr_level(level,1), max );
		output_tag( outptr, lvl2indent(level), "relatedItem", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
	}
	/* Look for original item things */
	orig_level = original_items( f, level );
	if ( orig_level ) {
		output_tag( outptr, lvl2indent(level), "relatedItem", NULL, TAG_OPEN,  TAG_NEWLINE, "type", "original", NULL );
		output_citeparts( f, outptr, orig_level, max );
		output_tag( outptr, lvl2indent(level), "relatedItem", NULL, TAG_CLOSE, TAG_NEWLINE, NULL );
	}
	output_abs(        f, outptr, level );
	output_notes(      f, outptr, level );
	output_toc(        f, outptr, level );
	output_key(        f, outptr, level );
	output_sn(         f, outptr, level );
	output_url(        f, outptr, level );
	output_part(       f, outptr, level );

	output_recordInfo( f, outptr, level );
}

static void
modsout_report_unused_tags( fields *f, param *p, unsigned long numrefs )
{
	int i, n, nwritten, nunused = 0, level;
	char *tag, *value;
	n = fields_num( f );
	for ( i=0; i<n; ++i ) {
		if ( fields_used( f, i ) ) continue;
		nunused++;
	}
	if ( nunused ) {
		if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
		fprintf( stderr, "Reference %lu has unused tags.\n", numrefs+1 );
		/* Find author from level 0 */
		nwritten = 0;
		for ( i=0; i<n; ++i ) {
			if ( fields_level( f, i ) != 0 ) continue;
			tag = fields_tag( f, i, FIELDS_CHRP_NOUSE );
			if ( strcasecmp( tag, "AUTHOR" ) && strcasecmp( tag, "AUTHOR:ASIS" ) && strcasecmp( tag, "AUTHOR:CORP" ) ) continue;
			value = fields_value( f, i, FIELDS_CHRP_NOUSE );
			if ( nwritten==0 ) fprintf( stderr, "\tAuthor(s) (level=0):\n" );
			fprintf( stderr, "\t\t'%s'\n", value );
			nwritten++;
		}
		nwritten = 0;
		for ( i=0; i<n; ++i ) {
			if ( fields_level( f, i ) != 0 ) continue;
			tag = fields_tag( f, i, FIELDS_CHRP_NOUSE );
			if ( strcasecmp( tag, "DATE:YEAR" ) && strcasecmp( tag, "PARTDATE:YEAR" ) ) continue;
			value = fields_value( f, i, FIELDS_CHRP_NOUSE );
			if ( nwritten==0 ) fprintf( stderr, "\tYear(s) (level=0):\n" );
			fprintf( stderr, "\t\t'%s'\n", value );
			nwritten++;
		}
		nwritten = 0;
		for ( i=0; i<n; ++i ) {
			if ( fields_level( f, i ) != 0 ) continue;
			tag = fields_tag( f, i, FIELDS_CHRP_NOUSE );
			if ( strncasecmp( tag, "TITLE", 5 ) ) continue;
			value = fields_value( f, i, FIELDS_CHRP_NOUSE );
			if ( nwritten==0 ) fprintf( stderr, "\tTitle(s) (level=0):\n" );
			fprintf( stderr, "\t\t'%s'\n", value );
			nwritten++;
		}
	
		fprintf( stderr, "\tUnused tags:\n" );
		for ( i=0; i<n; ++i ) {
			if ( fields_used( f, i ) ) continue;
			tag   = fields_tag(   f, i, FIELDS_CHRP_NOUSE );
			value = fields_value( f, i, FIELDS_CHRP_NOUSE );
			level = fields_level( f, i );
			fprintf( stderr, "\t\ttag: '%s' value: '%s' level: %d\n",
				tag, value, level );
		}
	}
}

static int
modsout_write( fields *f, FILE *outptr, param *p, unsigned long numrefs )
{
	int max, dropkey;
	max = fields_maxlevel( f );
	dropkey = ( p->format_opts & BIBL_FORMAT_MODSOUT_DROPKEY );

	output_head( f, outptr, dropkey, numrefs );
	output_citeparts( f, outptr, 0, max );
	modsout_report_unused_tags( f, p, numrefs );

	fprintf( outptr, "</mods>\n" );
	fflush( outptr );

	return BIBL_OK;
}

/*****************************************************
 PUBLIC: int modsout_writeheader()
*****************************************************/

static void
modsout_writeheader( FILE *outptr, param *p )
{
	if ( p->utf8bom ) utf8_writebom( outptr );
	fprintf(outptr,"<?xml version=\"1.0\" encoding=\"%s\"?>\n",
			charset_get_xmlname( p->charsetout ) );
	fprintf(outptr,"<modsCollection xmlns=\"http://www.loc.gov/mods/v3\">\n");
}

/*****************************************************
 PUBLIC: int modsout_writefooter()
*****************************************************/

static void
modsout_writefooter( FILE *outptr )
{
	fprintf(outptr,"</modsCollection>\n");
	fflush( outptr );
}

