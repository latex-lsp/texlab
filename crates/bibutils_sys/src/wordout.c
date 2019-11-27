/*
 * wordout.c
 * 
 * (Word 2007 format)
 *
 * Copyright (c) Chris Putnam 2007-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "str.h"
#include "fields.h"
#include "utf8.h"
#include "bibformats.h"

/*****************************************************
 PUBLIC: int wordout_initparams()
*****************************************************/

static void wordout_writeheader( FILE *outptr, param *p );
static void wordout_writefooter( FILE *outptr );
static int  wordout_write( fields *info, FILE *outptr, param *p, unsigned long numrefs );

int
wordout_initparams( param *pm, const char *progname )
{
	pm->writeformat      = BIBL_WORD2007OUT;
	pm->format_opts      = 0;
	pm->charsetout       = BIBL_CHARSET_UNICODE;
	pm->charsetout_src   = BIBL_SRC_DEFAULT;
	pm->latexout         = 0;
	pm->utf8out          = BIBL_CHARSET_UTF8_DEFAULT;
	pm->utf8bom          = BIBL_CHARSET_BOM_DEFAULT;
	if ( !pm->utf8out )
		pm->xmlout   = BIBL_XMLOUT_ENTITIES;
	else
		pm->xmlout   = BIBL_XMLOUT_TRUE;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->singlerefperfile = 0;

	pm->headerf   = wordout_writeheader;
	pm->footerf   = wordout_writefooter;
	pm->assemblef = NULL;
	pm->writef    = wordout_write;

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
 PUBLIC: int wordout_write()
*****************************************************/

typedef struct convert {
	char *oldtag;
	char *newtag;
	char *prefix;
	int  code;
} convert;

/*
At the moment 17 unique types of sources are defined:

{code}
	Art
	ArticleInAPeriodical
	Book
	BookSection
	Case
	Conference
	DocumentFromInternetSite
	ElectronicSource
	Film
	InternetSite
	Interview
	JournalArticle
	Report
	Misc
	Patent
	Performance
	Proceedings
	SoundRecording
{code}

*/

enum {
	TYPE_UNKNOWN = 0,
	TYPE_ART,
	TYPE_ARTICLEINAPERIODICAL,
	TYPE_BOOK,
	TYPE_BOOKSECTION,
	TYPE_CASE,
	TYPE_CONFERENCE,
	TYPE_DOCUMENTFROMINTERNETSITE,
	TYPE_ELECTRONICSOURCE,
	TYPE_FILM,
	TYPE_INTERNETSITE,
	TYPE_INTERVIEW,
	TYPE_JOURNALARTICLE,
	TYPE_MISC,
	TYPE_PATENT,
	TYPE_PERFORMANCE,
	TYPE_PROCEEDINGS,
	TYPE_REPORT,
	TYPE_SOUNDRECORDING,

	TYPE_THESIS,
	TYPE_MASTERSTHESIS,
	TYPE_PHDTHESIS,
};

/*
 * fixed output
 */
static void
output_fixed( FILE *outptr, char *tag, char *value, int level )
{
	int i;
	for ( i=0; i<level; ++i ) fprintf( outptr, " " );
	fprintf( outptr, "<%s>%s</%s>\n", tag, value, tag );
}

/* detail output
 *
 */
static void
output_item( fields *info, FILE *outptr, char *tag, char *prefix, int item, int level )
{
	int i;
	if ( item==-1 ) return;
	for ( i=0; i<level; ++i ) fprintf( outptr, " " );
	fprintf( outptr, "<%s>%s%s</%s>\n",
		tag,
		prefix,
		(char*) fields_value( info, item, FIELDS_CHRP ),
		tag
	);
}

static void
output_itemv( FILE *outptr, char *tag, char *item, int level )
{
	int i;
	for ( i=0; i<level; ++i ) fprintf( outptr, " " );
	fprintf( outptr, "<%s>%s</%s>\n", tag, item, tag );
}

/* range output
 *
 * <TAG>start-end</TAG>
 *
 */
static void
output_range( FILE *outptr, char *tag, char *start, char *end, int level )
{
	int i;
	if ( start==NULL && end==NULL ) return;
	if ( start==NULL )
		output_itemv( outptr, tag, end, 0 );
	else if ( end==NULL )
		output_itemv( outptr, tag, start, 0 );
	else {
		for ( i=0; i<level; ++i )
			fprintf( outptr, " " );
		fprintf( outptr, "<%s>%s-%s</%s>\n", tag, start, end, tag );
	}
}

static void
output_list( fields *info, FILE *outptr, convert *c, int nc )
{
        int i, n;
        for ( i=0; i<nc; ++i ) {
                n = fields_find( info, c[i].oldtag, c[i].code );
                if ( n!=FIELDS_NOTFOUND ) output_item( info, outptr, c[i].newtag, c[i].prefix, n, 0 );
        }

}

typedef struct outtype {
	int value;
	char *out;
} outtype;

static
outtype genres[] = {
	{ TYPE_PATENT,           "patent" },
	{ TYPE_REPORT,           "report" },
	{ TYPE_REPORT,           "technical report" },
	{ TYPE_CASE,             "legal case and case notes" },
	{ TYPE_ART,              "art original" },
	{ TYPE_ART,              "art reproduction" },
	{ TYPE_ART,              "comic strip" },
	{ TYPE_ART,              "diorama" },
	{ TYPE_ART,              "graphic" },
	{ TYPE_ART,              "model" },
	{ TYPE_ART,              "picture" },
	{ TYPE_ELECTRONICSOURCE, "electronic" },
	{ TYPE_FILM,             "videorecording" },
	{ TYPE_FILM,             "motion picture" },
	{ TYPE_SOUNDRECORDING,   "sound" },
	{ TYPE_PERFORMANCE,      "rehersal" },
	{ TYPE_INTERNETSITE,     "web site" },
	{ TYPE_INTERVIEW,        "interview" },
	{ TYPE_INTERVIEW,        "communication" },
	{ TYPE_MISC,             "misc" },
};
int ngenres = sizeof( genres ) / sizeof( genres[0] );

static int
get_type_from_genre( fields *info )
{
	int type = TYPE_UNKNOWN, i, j, level;
	char *genre, *tag;
	for ( i=0; i<info->n; ++i ) {
		tag = (char *) fields_tag( info, i, FIELDS_CHRP );
		if ( strcasecmp( tag, "GENRE:MARC" ) && strcasecmp( tag, "GENRE:BIBUTILS" ) && strcasecmp( tag, "GENRE:UNKNOWN" ) ) continue;
		genre = (char *) fields_value( info, i, FIELDS_CHRP );
		for ( j=0; j<ngenres; ++j ) {
			if ( !strcasecmp( genres[j].out, genre ) )
				type = genres[j].value;
		}
		if ( type==TYPE_UNKNOWN ) {
			level = info->level[i];
			if ( !strcasecmp( genre, "academic journal" ) ) {
				type = TYPE_JOURNALARTICLE;
			}
			else if ( !strcasecmp( genre, "periodical" ) ) {
				if ( type == TYPE_UNKNOWN )
					type = TYPE_ARTICLEINAPERIODICAL;
			}
			else if ( !strcasecmp( genre, "book" ) ||
				!strcasecmp( genre, "collection" ) ) {
				if ( info->level[i]==0 ) type = TYPE_BOOK;
				else type = TYPE_BOOKSECTION;
			}
			else if ( !strcasecmp( genre, "conference publication" ) ) {
				if ( level==0 ) type=TYPE_CONFERENCE;
				else type = TYPE_PROCEEDINGS;
			}
			else if ( !strcasecmp( genre, "thesis" ) ) {
	                        if ( type==TYPE_UNKNOWN ) type=TYPE_THESIS;
			}
			else if ( !strcasecmp( genre, "Ph.D. thesis" ) ) {
				type = TYPE_PHDTHESIS;
			}
			else if ( !strcasecmp( genre, "Masters thesis" ) ) {
				type = TYPE_MASTERSTHESIS;
			}
		}
	}
	return type;
}

static int
get_type_from_resource( fields *info )
{
	int type = TYPE_UNKNOWN, i;
	char *tag, *resource;
	for ( i=0; i<info->n; ++i ) {
		tag = (char *) fields_tag( info, i, FIELDS_CHRP );
		if ( strcasecmp( tag, "RESOURCE" ) ) continue;
		resource = (char *) fields_value( info, i, FIELDS_CHRP );
		if ( !strcasecmp( resource, "moving image" ) )
			type = TYPE_FILM;
	}
	return type;
}

static int
get_type( fields *info )
{
	int type;
	type = get_type_from_genre( info );
	if ( type==TYPE_UNKNOWN )
		type = get_type_from_resource( info );
	return type;
}

static void
output_titlebits( char *mainttl, char *subttl, FILE *outptr )
{
	if ( mainttl ) fprintf( outptr, "%s", mainttl );
	if ( subttl ) {
		if ( mainttl ) {
			if ( mainttl[ strlen( mainttl ) - 1 ] != '?' )
				fprintf( outptr, ": " );
			else fprintf( outptr, " " );
		}
		fprintf( outptr, "%s", subttl );
	}
}

static void
output_titleinfo( char *mainttl, char *subttl, FILE *outptr, char *tag, int level )
{
	if ( mainttl || subttl ) {
		fprintf( outptr, "<%s>", tag );
		output_titlebits( mainttl, subttl, outptr );
		fprintf( outptr, "</%s>\n", tag );
	}
}

static void
output_generaltitle( fields *info, FILE *outptr, char *tag, int level )
{
	char *ttl       = fields_findv( info, level, FIELDS_CHRP, "TITLE" );
	char *subttl    = fields_findv( info, level, FIELDS_CHRP, "SUBTITLE" );
	char *shrttl    = fields_findv( info, level, FIELDS_CHRP, "SHORTTITLE" );
	char *shrsubttl = fields_findv( info, level, FIELDS_CHRP, "SHORTSUBTITLE" );

	if ( ttl ) {
		output_titleinfo( ttl, subttl, outptr, tag, level );
	}
	else if ( shrttl ) {
		output_titleinfo( shrttl, shrsubttl, outptr, tag, level );
	}
}

static void
output_maintitle( fields *info, FILE *outptr, int level )
{
	char *ttl       = fields_findv( info, level, FIELDS_CHRP, "TITLE" );
	char *subttl    = fields_findv( info, level, FIELDS_CHRP, "SUBTITLE" );
	char *shrttl    = fields_findv( info, level, FIELDS_CHRP, "SHORTTITLE" );
	char *shrsubttl = fields_findv( info, level, FIELDS_CHRP, "SHORTSUBTITLE" );

	if ( ttl ) {
		output_titleinfo( ttl, subttl, outptr, "b:Title", level );

		/* output shorttitle if it's different from normal title */
		if ( shrttl ) {
			if ( !ttl || ( strcmp( shrttl, ttl ) || subttl ) ) {
				fprintf( outptr,  " <b:ShortTitle>" );
				output_titlebits( shrttl, shrsubttl, outptr );
				fprintf( outptr, "</b:ShortTitle>\n" );
			}
		}
	}
	else if ( shrttl ) {
		output_titleinfo( shrttl, shrsubttl, outptr, "b:Title", level );
	}
}

static void
output_name_nomangle( FILE *outptr, char *p )
{
	fprintf( outptr, "<b:Person>" );
	fprintf( outptr, "<b:Last>%s</b:Last>", p );
	fprintf( outptr, "</b:Person>\n" );
}

static void
output_name( FILE *outptr, char *p )
{
	str family, part;
	int n=0, npart=0;

	str_init( &family );
	while ( *p && *p!='|' ) str_addchar( &family, *p++ );
	if ( *p=='|' ) p++;
	if ( str_has_value( &family ) ) {
		fprintf( outptr, "<b:Person>" );
		fprintf( outptr, "<b:Last>%s</b:Last>", str_cstr( &family ) );
		n++;
	}
	str_free( &family );

	str_init( &part );
	while ( *p ) {
		while ( *p && *p!='|' ) str_addchar( &part, *p++ );
		if ( str_has_value( &part ) ) {
			if ( n==0 ) fprintf( outptr, "<b:Person>" );
			if ( npart==0 ) 
				fprintf( outptr, "<b:First>%s</b:First>", str_cstr( &part ) );
			else
				fprintf( outptr, "<b:Middle>%s</b:Middle>", str_cstr( &part ) );
			n++;
			npart++;
		}
		if ( *p=='|' ) {
			p++;
			str_empty( &part );
		}
	}
	if ( n ) fprintf( outptr, "</b:Person>\n" );

	str_free( &part );
}


#define NAME (1)
#define NAME_ASIS (2)
#define NAME_CORP (4)

static int
extract_name_and_info( str *outtag, str *intag )
{
	int code = NAME;
	str_strcpy( outtag, intag );
	if ( str_findreplace( outtag, ":ASIS", "" ) ) code = NAME_ASIS;
	if ( str_findreplace( outtag, ":CORP", "" ) ) code = NAME_CORP;
	return code;
}

static void
output_name_type( fields *info, FILE *outptr, int level, 
			char *map[], int nmap, char *tag )
{
	str ntag;
	int i, j, n=0, code, nfields;
	str_init( &ntag );
	nfields = fields_num( info );
	for ( j=0; j<nmap; ++j ) {
		for ( i=0; i<nfields; ++i ) {
			code = extract_name_and_info( &ntag, &(info->tag[i]) );
			if ( strcasecmp( str_cstr( &ntag ), map[j] ) ) continue;
			if ( n==0 )
				fprintf( outptr, "<%s><b:NameList>\n", tag );
			if ( code != NAME )
				output_name_nomangle( outptr, (char *) fields_value( info, i, FIELDS_CHRP ) );
			else 
				output_name( outptr, (char *) fields_value( info, i, FIELDS_CHRP ) );
			n++;
		}
	}
	str_free( &ntag );
	if ( n )
		fprintf( outptr, "</b:NameList></%s>\n", tag );
}

static void
output_names( fields *info, FILE *outptr, int level, int type )
{
	char *authors[] = { "AUTHOR", "WRITER", "ASSIGNEE", "ARTIST",
		"CARTOGRAPHER", "INVENTOR", "ORGANIZER", "DIRECTOR",
		"PERFORMER", "REPORTER", "TRANSLATOR", "ADDRESSEE",
		"2ND_AUTHOR", "3RD_AUTHOR", "SUB_AUTHOR", "COMMITTEE",
		"COURT", "LEGISLATIVEBODY" };
	int nauthors = sizeof( authors ) / sizeof( authors[0] );

	char *editors[] = { "EDITOR" };
	int neditors = sizeof( editors ) / sizeof( editors[0] );

	char author_default[] = "b:Author", inventor[] = "b:Inventor";
	char *author_type = author_default;

	if ( type == TYPE_PATENT ) author_type = inventor;

	fprintf( outptr, "<b:Author>\n" );
	output_name_type( info, outptr, level, authors, nauthors, author_type );
	output_name_type( info, outptr, level, editors, neditors, "b:Editor" );
	fprintf( outptr, "</b:Author>\n" );
}

static void
output_date( fields *info, FILE *outptr, int level )
{
	char *year  = fields_findv_firstof( info, level, FIELDS_CHRP,
			"PARTDATE:YEAR", "DATE:YEAR", NULL );
	char *month = fields_findv_firstof( info, level, FIELDS_CHRP,
			"PARTDATE:MONTH", "DATE:MONTH", NULL );
	char *day   = fields_findv_firstof( info, level, FIELDS_CHRP,
			"PARTDATE:DAY", "DATE:DAY", NULL );
	if ( year )  output_itemv( outptr, "b:Year", year, 0 );
	if ( month ) output_itemv( outptr, "b:Month", month, 0 );
	if ( day )   output_itemv( outptr, "b:Day", day, 0 );
}

static void
output_pages( fields *info, FILE *outptr, int level )
{
	char *sn = fields_findv( info, LEVEL_ANY, FIELDS_CHRP, "PAGES:START" );
	char *en = fields_findv( info, LEVEL_ANY, FIELDS_CHRP, "PAGES:STOP" );
	char *ar = fields_findv( info, LEVEL_ANY, FIELDS_CHRP, "ARTICLENUMBER" );
	if ( sn || en )
		output_range( outptr, "b:Pages", sn, en, level );
	else if ( ar )
		output_range( outptr, "b:Pages", ar, NULL, level );
}

static void
output_includedin( fields *info, FILE *outptr, int type )
{
	if ( type==TYPE_JOURNALARTICLE ) {
		output_generaltitle( info, outptr, "b:JournalName", 1 );
	} else if ( type==TYPE_ARTICLEINAPERIODICAL ) {
		output_generaltitle( info, outptr, "b:PeriodicalTitle", 1 );
	} else if ( type==TYPE_BOOKSECTION ) {
		output_generaltitle( info, outptr, "b:ConferenceName", 1 ); /*??*/
	} else if ( type==TYPE_PROCEEDINGS ) {
		output_generaltitle( info, outptr, "b:ConferenceName", 1 );
	}
}

static int
type_is_thesis( int type )
{
	if ( type==TYPE_THESIS ||
	     type==TYPE_PHDTHESIS ||
	     type==TYPE_MASTERSTHESIS )
		return 1;
	else
		return 0;
}

static void
output_thesisdetails( fields *info, FILE *outptr, int type )
{
	char *tag;
	int i, n;

	if ( type==TYPE_PHDTHESIS )
		output_fixed( outptr, "b:ThesisType", "Ph.D. Thesis", 0 );
	else if ( type==TYPE_MASTERSTHESIS ) 
		output_fixed( outptr, "b:ThesisType", "Masters Thesis", 0 );

	n = fields_num( info );
	for ( i=0; i<n; ++i ) {
		tag = fields_tag( info, i, FIELDS_CHRP );
		if ( strcasecmp( tag, "DEGREEGRANTOR" ) &&
			strcasecmp( tag, "DEGREEGRANTOR:ASIS") &
			strcasecmp( tag, "DEGREEGRANTOR:CORP"))
				continue;
		output_item( info, outptr, "b:Institution", "", i, 0 );
	}
}

static
outtype types[] = {
	{ TYPE_UNKNOWN,                  "Misc" },
	{ TYPE_MISC,                     "Misc" },
	{ TYPE_BOOK,                     "Book" },
	{ TYPE_BOOKSECTION,              "BookSection" },
	{ TYPE_CASE,                     "Case" },
	{ TYPE_CONFERENCE,               "Conference" },
	{ TYPE_ELECTRONICSOURCE,         "ElectronicSource" },
	{ TYPE_FILM,                     "Film" },
	{ TYPE_INTERNETSITE,             "InternetSite" },
	{ TYPE_INTERVIEW,                "Interview" },
	{ TYPE_SOUNDRECORDING,           "SoundRecording" },
	{ TYPE_ARTICLEINAPERIODICAL,     "ArticleInAPeriodical" },
	{ TYPE_DOCUMENTFROMINTERNETSITE, "DocumentFromInternetSite" },
	{ TYPE_JOURNALARTICLE,           "JournalArticle" },
	{ TYPE_REPORT,                   "Report" },
	{ TYPE_PATENT,                   "Patent" },
	{ TYPE_PERFORMANCE,              "Performance" },
	{ TYPE_PROCEEDINGS,              "Proceedings" },
};
static
int ntypes = sizeof( types ) / sizeof( types[0] );

static void
output_type( fields *info, FILE *outptr, int type )
{
	int i, found = 0;
	fprintf( outptr, "<b:SourceType>" );
	for ( i=0; i<ntypes && !found; ++i ) {
		if ( types[i].value!=type ) continue;
		found = 1;
		fprintf( outptr, "%s", types[i].out );
	}
	if ( !found ) {
		if (  type_is_thesis( type ) ) fprintf( outptr, "Report" );
		else fprintf( outptr, "Misc" );
	}
	fprintf( outptr, "</b:SourceType>\n" );

	if ( type_is_thesis( type ) )
		output_thesisdetails( info, outptr, type );
}

static void
output_comments( fields *info, FILE *outptr, int level )
{
	vplist_index i;
	vplist notes;
	char *abs;

	vplist_init( &notes );

	abs = fields_findv( info, level, FIELDS_CHRP, "ABSTRACT" );
	fields_findv_each( info, level, FIELDS_CHRP, &notes, "NOTES" );

	if ( abs || notes.n ) fprintf( outptr, "<b:Comments>" );
	if ( abs ) fprintf( outptr, "%s", abs );
	for ( i=0; i<notes.n; ++i )
		fprintf( outptr, "%s", (char*)vplist_get( &notes, i ) );
	if ( abs || notes.n ) fprintf( outptr, "</b:Comments>\n" );

	vplist_free( &notes );
}

static void
output_bibkey( fields *info, FILE *outptr )
{
	char *bibkey = fields_findv_firstof( info, LEVEL_ANY, FIELDS_CHRP,
			"REFNUM", "BIBKEY", NULL );
	if ( bibkey ) output_itemv( outptr, "b:Tag", bibkey, 0 );
}

static void
output_citeparts( fields *info, FILE *outptr, int level, int max, int type )
{
	convert origin[] = {
		{ "ADDRESS",	"b:City",	"", LEVEL_ANY },
		{ "PUBLISHER",	"b:Publisher",	"", LEVEL_ANY },
		{ "EDITION",	"b:Edition",	"", LEVEL_ANY }
	};
	int norigin = sizeof( origin ) / sizeof ( convert );
	
	convert parts[] = {
		{ "VOLUME",          "b:Volume",  "", LEVEL_ANY },
		{ "SECTION",         "b:Section", "", LEVEL_ANY },
		{ "ISSUE",           "b:Issue",   "", LEVEL_ANY },
		{ "NUMBER",          "b:Issue",   "", LEVEL_ANY },
		{ "PUBLICLAWNUMBER", "b:Volume",  "", LEVEL_ANY },
		{ "SESSION",         "b:Issue",   "", LEVEL_ANY },
		{ "URL",             "b:Url",     "", LEVEL_ANY },
		{ "JSTOR",           "b:Url",     "http://www.jstor.org/stable/", LEVEL_ANY },
		{ "ARXIV",           "b:Url",     "http://arxiv.org/abs/",        LEVEL_ANY },
		{ "PMID",            "b:Url",     "http://www.ncbi.nlm.nih.gov/pubmed/", LEVEL_ANY },
		{ "PMC",             "b:Url",     "http://www.ncbi.nlm.nih.gov/pmc/articles/", LEVEL_ANY },
		{ "DOI",             "b:Url",     "https://doi.org/", LEVEL_ANY },
		{ "MRNUMBER",        "b:Url",     "http://www.ams.org/mathscinet-getitem?mr=", LEVEL_ANY },
	};
	int nparts=sizeof(parts)/sizeof(convert);
	
	output_bibkey( info, outptr );
	output_type( info, outptr, type );
	output_list( info, outptr, origin, norigin );
	output_date( info, outptr, level );
	output_includedin( info, outptr, type );
	output_list( info, outptr, parts, nparts );
	output_pages( info, outptr, level );
	output_names( info, outptr, level, type );
	output_maintitle( info, outptr, 0 );
	output_comments( info, outptr, level );
}

static int
wordout_write( fields *info, FILE *outptr, param *p, unsigned long numrefs )
{
	int max = fields_maxlevel( info );
	int type = get_type( info );

	fprintf( outptr, "<b:Source>\n" );
	output_citeparts( info, outptr, -1, max, type );
	fprintf( outptr, "</b:Source>\n" );

	fflush( outptr );

	return BIBL_OK;
}

/*****************************************************
 PUBLIC: void wordout_writeheader()
*****************************************************/

static void
wordout_writeheader( FILE *outptr, param *p )
{
	if ( p->utf8bom ) utf8_writebom( outptr );
	fprintf(outptr,"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
	fprintf(outptr,"<b:Sources SelectedStyle=\"\" "
		"xmlns:b=\"http://schemas.openxmlformats.org/officeDocument/2006/bibliography\" "
		" xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/bibliography\" >\n");
}

/*****************************************************
 PUBLIC: void wordout_writefooter()
*****************************************************/

static void
wordout_writefooter( FILE *outptr )
{
	fprintf(outptr,"</b:Sources>\n");
	fflush( outptr );
}
