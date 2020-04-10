/*
 * mycvout.c
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
#include "strsearch.h"
#include "utf8.h"
#include "xml.h"
#include "fields.h"
#include "bibl.h"
#include "doi.h"
#include "bibutils.h"

void
bibtexout_initparams( param *pm, const char *progname )
{
	pm->writeformat      = BIBL_BIBTEXOUT;
	pm->format_opts      = 0;
	pm->charsetout       = BIBL_CHARSET_DEFAULT;
	pm->charsetout_src   = BIBL_SRC_DEFAULT;
	pm->latexout         = 1;
	pm->utf8out          = 0;
	pm->utf8bom          = 0;
	pm->xmlout           = BIBL_XMLOUT_FALSE;
	pm->nosplittitle     = 0;
	p->verbose          = 0;
	pm->addcount         = 0;
	pm->singlerefperfile = 0;

	pm->headerf = bibtexout_writeheader;
	pm->footerf = NULL;
	pm->writef  = bibtexout_write;

	if ( !p->progname && progname )
		p->progname = strdup( progname );
}

enum {
	TYPE_UNKNOWN = 0,
	TYPE_ARTICLE,
	TYPE_INBOOK,
	TYPE_INPROCEEDINGS,
	TYPE_PROCEEDINGS,
	TYPE_INCOLLECTION,
	TYPE_COLLECTION,
	TYPE_BOOK,
	TYPE_PHDTHESIS,
	TYPE_MASTERSTHESIS,
	TYPE_REPORT,
	TYPE_MANUAL,
	TYPE_UNPUBLISHED,
	TYPE_ELECTRONIC,
	TYPE_MISC
};

static void
output_citekey( FILE *fp, fields *info, unsigned long refnum, int format_opts )
{
	int n = fields_find( info, "REFNUM", -1 );
	char *p;
	if ( n!=-1 ) {
		p = info->data[n].data;
		while ( p && *p && *p!='|' ) {
			if ( format_opts & BIBL_FORMAT_BIBOUT_STRICTKEY ) {
				if ( isdigit((unsigned char)*p) || (*p>='A' && *p<='Z') ||
				     (*p>='a' && *p<='z' ) )
					fprintf( fp, "%c", *p );
			}
			else {
				if ( *p!=' ' && *p!='\t' ) {
					fprintf( fp, "%c", *p );
				}
			}
			p++;
		}
	}
}

static int
bibtexout_type( fields *info, char *filename, int refnum, param *p )
{
	char *genre;
	int type = TYPE_UNKNOWN, i, maxlevel, n, level;

	/* determine bibliography type */
	for ( i=0; i<info->n; ++i ) {
		if ( strcasecmp( info->tag[i].data, "GENRE:MARC" ) &&
		     strcasecmp( info->tag[i].data, "GENRE:BIBUTILS" ) ) continue;
		genre = info->data[i].data;
		level = info->level[i];
		if ( !strcasecmp( genre, "periodical" ) ||
		     !strcasecmp( genre, "academic journal" ) ||
		     !strcasecmp( genre, "magazine" ) ||
		     !strcasecmp( genre, "newspaper" ) ||
		     !strcasecmp( genre, "article" ) )
			type = TYPE_ARTICLE;
		else if ( !strcasecmp( genre, "instruction" ) )
			type = TYPE_MANUAL;
		else if ( !strcasecmp( genre, "unpublished" ) )
			type = TYPE_UNPUBLISHED;
		else if ( !strcasecmp( genre, "conference publication" ) ) {
			if ( level==0 ) type=TYPE_PROCEEDINGS;
			else type = TYPE_INPROCEEDINGS;
		} else if ( !strcasecmp( genre, "collection" ) ) {
			if ( level==0 ) type=TYPE_COLLECTION;
			else type = TYPE_INCOLLECTION;
		} else if ( !strcasecmp( genre, "technical report" ) )
			type = TYPE_REPORT;
		else if ( !strcasecmp( genre, "book" ) ) {
			if ( level==0 ) type=TYPE_BOOK;
			else type=TYPE_INBOOK;
		} else if ( !strcasecmp( genre, "thesis" ) ) {
			if ( type==TYPE_UNKNOWN ) type=TYPE_PHDTHESIS;
		} else if ( !strcasecmp( genre, "Ph.D. thesis" ) )
			type = TYPE_PHDTHESIS;
		else if ( !strcasecmp( genre, "Masters thesis" ) )
			type = TYPE_MASTERSTHESIS;
		else  if ( !strcasecmp( genre, "electronic" ) )
			type = TYPE_ELECTRONIC;
	}
	if ( type==TYPE_UNKNOWN ) {
		for ( i=0; i<info->n; ++i ) {
			if ( strcasecmp( info->tag[i].data, "ISSUANCE" ) ) continue;
			if ( !strcasecmp( info->data[i].data, "monographic" ) ) {
				if ( info->level[i]==0 ) type = TYPE_BOOK;
				else if ( info->level[i]==1 ) type=TYPE_INBOOK;
			}
		}
	}

	/* default to BOOK type */
	if ( type==TYPE_UNKNOWN ) {
		maxlevel = fields_maxlevel( info );
		if ( maxlevel > 0 ) type = TYPE_INBOOK;
		else {
			if ( p->progname ) fprintf( stderr, "%s: ", p->progname );
			fprintf( stderr, "Cannot identify TYPE "
				"in reference %d ", refnum+1 );
			n = fields_find( info, "REFNUM", -1 );
			if ( n!=-1 ) 
				fprintf( stderr, " %s", info->data[n].data);
			fprintf( stderr, " (defaulting to @Misc)\n" );
			type = TYPE_MISC;
		}
	}
	return type;
}

static void
output_type( FILE *fp, int type, int format_opts )
{
	typedef struct {
		int bib_type;
		char *type_name;
	} typenames;

	typenames types[] = {
		{ TYPE_ARTICLE, "Article" },
		{ TYPE_INBOOK, "Inbook" },
		{ TYPE_PROCEEDINGS, "Proceedings" },
		{ TYPE_INPROCEEDINGS, "InProceedings" },
		{ TYPE_BOOK, "Book" },
		{ TYPE_PHDTHESIS, "PhdThesis" },
		{ TYPE_MASTERSTHESIS, "MastersThesis" },
		{ TYPE_REPORT, "TechReport" },
		{ TYPE_MANUAL, "Manual" },
		{ TYPE_COLLECTION, "Collection" },
		{ TYPE_INCOLLECTION, "InCollection" },
		{ TYPE_UNPUBLISHED, "Unpublished" },
		{ TYPE_ELECTRONIC, "Electronic" },
		{ TYPE_MISC, "Misc" } };
	int i, len, ntypes = sizeof( types ) / sizeof( types[0] );
	char *s = NULL;
	for ( i=0; i<ntypes; ++i ) {
		if ( types[i].bib_type == type ) {
			s = types[i].type_name;
			break;
		}
	}
	if ( !s ) s = types[ntypes-1].type_name; /* default to TYPE_MISC */
	if ( !(format_opts & BIBL_FORMAT_BIBOUT_UPPERCASE ) ) fprintf( fp, "@%s{", s );
	else {
		len = strlen( s );
		fprintf( fp, "@" );
		for ( i=0; i<len; ++i )
			fprintf( fp, "%c", toupper((unsigned char)s[i]) );
		fprintf( fp, "{" );
	}
}

static void
output_element( FILE *fp, char *tag, char *data, int format_opts )
{
	int i, len, nquotes = 0;
	char ch;
	fprintf( fp, ",\n" );
	if ( format_opts & BIBL_FORMAT_BIBOUT_WHITESPACE ) fprintf( fp, "  " );
	if ( !(format_opts & BIBL_FORMAT_BIBOUT_UPPERCASE ) ) fprintf( fp, "%s", tag );
	else {
		len = strlen( tag );
		for ( i=0; i<len; ++i )
			fprintf( fp, "%c", toupper((unsigned char)tag[i]) );
	}
	if ( format_opts & BIBL_FORMAT_BIBOUT_WHITESPACE ) fprintf( fp, " = \t" );
	else fprintf( fp, "=" );

	if ( format_opts & BIBL_FORMAT_BIBOUT_BRACKETS ) fprintf( fp, "{" );
	else fprintf( fp, "\"" );

	len = strlen( data );
	for ( i=0; i<len; ++i ) {
		ch = data[i];
		if ( ch!='\"' ) fprintf( fp, "%c", ch );
		else {
			if ( format_opts & BIBL_FORMAT_BIBOUT_BRACKETS || 
			    ( i>0 && data[i-1]=='\\' ) )
				fprintf( fp, "\"" );
			else {
				if ( nquotes % 2 == 0 )
					fprintf( fp, "``" );
				else    fprintf( fp, "\'\'" );
				nquotes++;
			}
		}
	}

	if ( format_opts & BIBL_FORMAT_BIBOUT_BRACKETS ) fprintf( fp, "}" );
	else fprintf( fp, "\"" );
}

static void
output_and_use( FILE *fp, fields *info, int n, char *outtag, int format_opts )
{
	output_element( fp, outtag, info->data[n].data, format_opts );
	fields_setused( info, n );
}

static void
output_simple( FILE *fp, fields *info, char *intag, char *outtag, 
		int format_opts )
{
	int n = fields_find( info, intag, -1 );
	if ( n!=-1 ) {
		output_and_use( fp, info, n, outtag, format_opts );
	}
}

static void
output_simpleall( FILE *fp, fields *info, char *intag, char *outtag,
		int format_opts )
{
	int i;
	for ( i=0; i<info->n; ++i ) {
		if ( strcasecmp( info->tag[i].data, intag ) ) continue;
		output_and_use( fp, info, i, outtag, format_opts );
	}
}

static void
output_fileattach( FILE *fp, fields *info, int format_opts )
{
	str data;
	int i;
	str_init( &data );
	for ( i=0; i<info->n; ++i ) {
		if ( strcasecmp( info->tag[i].data, "FILEATTACH" ) ) continue;
		str_strcpy( &data, ":" );
		str_strcatc( &data, &(info->data[i]) );
		if ( strsearch( info->data[i].data, ".pdf" ) )
			str_strcatc( &data, ":PDF" );
		else if ( strsearch( info->data[i].data, ".html" ) )
			str_strcatc( &data, ":HTML" );
		else str_strcatc( &data, ":TYPE" );
		output_element( fp, "file", data.data, format_opts );
		fields_setused( info, i );
		str_empty( &data );
	}
	str_free( &data );
}

static void
add_person( str *s, char *p )
{
	int nseps = 0, nch;
	while ( *p ) {
		nch = 0;
		if ( nseps==1 ) str_addchar( s, ',' );
		if ( nseps ) str_addchar( s, ' ' );
		while ( *p && *p!='|' ) {
			str_addchar( s, *p++ );
			nch++;
		}
		if ( *p=='|' ) p++;
		if ( nseps!=0 && nch==1 ) str_addchar( s, '.' );
		nseps++;
	}
}

static void
output_people( FILE *fp, fields *info, unsigned long refnum, char *tag, 
		char *ctag, char *atag, char *bibtag, int level, 
		int format_opts )
{
	str allpeople;
	int i, npeople, person, corp, asis;

	/* primary citation authors */
	npeople = 0;
	for ( i=0; i<info->n; ++i ) {
		if ( level!=-1 && info->level[i]!=level ) continue;
		person = ( strcasecmp( info->tag[i].data, tag ) == 0 );
		corp   = ( strcasecmp( info->tag[i].data, ctag ) == 0 );
		asis   = ( strcasecmp( info->tag[i].data, atag ) == 0 );
		if ( person || corp || asis ) {
			if ( npeople==0 ) str_init( &allpeople );
			else {
				if ( format_opts & BIBL_FORMAT_BIBOUT_WHITESPACE )
					str_strcatc(&allpeople,"\n\t\tand ");
				else str_strcatc( &allpeople, "\nand " );
			}
			if ( corp ) {
				str_addchar( &allpeople, '{' );
				str_strcatc( &allpeople, info->data[i].data );
				str_addchar( &allpeople, '}' );
			} else if ( asis ) {
				str_addchar( &allpeople, '{' );
				str_strcatc( &allpeople, info->data[i].data );
				str_addchar( &allpeople, '}' );
			} else add_person( &allpeople, info->data[i].data ); 
			fields_setused( info, i );
			npeople++;
		}
	}
	if ( npeople ) {
		output_element( fp, bibtag, allpeople.data, format_opts );
		str_free( &allpeople );
	}
}

static void
output_title( FILE *fp, fields *info, unsigned long refnum, char *bibtag, int level, int format_opts )
{
	str title;
	int n1 = -1, n2 = -1;
	/* Option is for short titles of journals */
	if ( ( format_opts & BIBL_FORMAT_BIBOUT_SHORTTITLE ) && level==1 ) {
		n1 = fields_find( info, "SHORTTITLE", level );
		n2 = fields_find( info, "SHORTSUBTITLE", level );
	}
	if ( n1==-1 ) {
		n1 = fields_find( info, "TITLE", level );
		n2 = fields_find( info, "SUBTITLE", level );
	}
	if ( n1!=-1 ) {
		str_init( &title );
		str_strcpy( &title, &(info->data[n1]) );
		fields_setused( info, n1 );
		if ( n2!=-1 ) {
			if ( info->data[n1].data[info->data[n1].len]!='?' )
				str_strcatc( &title, ": " );
			else str_addchar( &title, ' ' );
			str_strcatc( &title, info->data[n2].data );
			fields_setused( info, n2 );
		}
		output_element( fp, bibtag, title.data, format_opts );
		str_free( &title );
	}
}

static void
output_date( FILE *fp, fields *info, unsigned long refnum, int format_opts )
{
	char *months[12] = { "Jan", "Feb", "Mar", "Apr", "May", "Jun", 
		"Jul", "Aug", "Sep", "Oct", "Nov", "Dec" };
	int n, month;
	n = fields_find( info, "DATE:YEAR", -1 );
	if ( n==-1 ) n = fields_find( info, "PARTDATE:YEAR", -1 );
	if ( n!=-1 ) {
		output_element( fp, "year", info->data[n].data, format_opts );
		fields_setused( info, n );
	}
	n = fields_find( info, "DATE:MONTH", -1 );
	if ( n==-1 ) n = fields_find( info, "PARTDATE:MONTH", -1 );
	if ( n!=-1 ) {
		month = atoi( info->data[n].data );
		if ( month>0 && month<13 )
			output_element( fp, "month", months[month-1], format_opts );
		else
			output_element( fp, "month", info->data[n].data, format_opts );
		fields_setused( info, n );
	}
	n = fields_find( info, "DATE:DAY", -1 );
	if ( n==-1 ) n = fields_find( info, "PARTDATE:DAY", -1 );
	if ( n!=-1 ) {
		output_element( fp, "day", info->data[n].data, format_opts );
		fields_setused( info, n );
	}
}


/* output article number as pages if true pages aren't found */
static void
output_articlenumber( FILE *fp, fields *info, unsigned long refnum,
	int format_opts )
{
	int ar = fields_find( info, "ARTICLENUMBER", -1 );
	if ( ar!=-1 ) {
		str pages;
		str_init( &pages );
		str_strcatc( &pages, info->data[ar].data );
		output_element( fp, "pages", pages.data, format_opts );
		fields_setused( info, ar );
		str_free( &pages );
	}
}

static void
output_arxiv( FILE *fp, fields *info, int format_opts )
{
	int ar = fields_find( info, "ARXIV", -1 );
	if ( ar!=-1 ) {
		str arxiv;
		str_init( &arxiv );
		arxiv_to_url( info, ar, "URL", &arxiv );
		if ( arxiv.len )
			output_element( fp, "url", arxiv.data, format_opts );
		str_free( &arxiv );
	}
}

static void
output_pmid( FILE *fp, fields *info, int format_opts )
{
	int pm = fields_find( info, "PMID", -1 );
	if ( pm!=-1 ) {
		str pmid;
		str_init( &pmid );
		pmid_to_url( info, pm, "URL", &pmid );
		if ( pmid.len )
			output_element( fp, "url", pmid.data, format_opts );
		str_free( &pmid );
	}
}

static void
output_jstor( FILE *fp, fields *info, int format_opts )
{
	int js = fields_find( info, "JSTOR", -1 );
	if ( js!=-1 ) {
		str jstor;
		str_init( &jstor );
		jstor_to_url( info, js, "URL", &jstor );
		if ( jstor.len )
			output_element( fp, "url", jstor.data, format_opts );
		str_free( &jstor );
	}
}

static void
output_pages( FILE *fp, fields *info, unsigned long refnum, int format_opts )
{
	str pages;
	int sn, en;
	sn = fields_find( info, "PAGES:START", -1 );
	en = fields_find( info, "PAGES:STOP", -1 );
	if ( sn==-1 && en==-1 ) {
		output_articlenumber( fp, info, refnum, format_opts );
		return;
	}
	str_init( &pages );
	if ( sn!=-1 ) {
		str_strcat( &pages, info->data[sn].data );
		fields_setused( info, sn );
	}
	if ( sn!=-1 && en!=-1 ) {
		if ( format_opts & BIBL_FORMAT_BIBOUT_SINGLEDASH ) 
			str_strcat( &pages, "-" );
		else
			str_strcat( &pages, "--" );
	}
	if ( en!=-1 ) {
		str_strcat( &pages, info->data[en].data );
		fields_setused( info, en );
	}
	output_element( fp, "pages", pages.data, format_opts );
	str_free( &pages );
}

/*
 * from Tim Hicks:
 * I'm no expert on bibtex, but those who know more than I on our mailing 
 * list suggest that 'issue' isn't a recognised key for bibtex and 
 * therefore that bibutils should be aliasing IS to number at some point in 
 * the conversion.
 *
 * Therefore prefer outputting issue/number as number and only keep
 * a distinction if both issue and number are present for a particular
 * reference.
 */

static void
output_issue_number( FILE *fp, fields *info, int format_opts )
{
	int nissue  = fields_find( info, "ISSUE", -1 );
	int nnumber = fields_find( info, "NUMBER", -1 );
	if ( nissue!=-1 && nnumber!=-1 ) {
		output_and_use( fp, info, nissue,  "issue",  format_opts );
		output_and_use( fp, info, nnumber, "number", format_opts );
/*		output_element( fp, "issue", info->data[nissue].data, 
				format_opts );
		fields_setused( info, nissue );
		output_element( fp, "number", info->data[nnumber].data, 
				format_opts );
		fields_setused( info, nnumber );*/
	} else if ( nissue!=-1 ) {
		output_and_use( fp, info, nissue, "number", format_opts );
/*
		output_element( fp, "number", info->data[nissue].data, 
				format_opts );
		fields_setused( info, nissue );*/
	} else if ( nnumber!=-1 ) {
		output_and_use( fp, info, nnumber, "number", format_opts );
/*
		output_element( fp, "number", info->data[nnumber].data, 
				format_opts );
		fields_setused( info, nnumber );
*/
	}
}

void
bibtexout_write( fields *info, FILE *fp, param *p, unsigned long refnum )
{
	int type;
	fields_clearused( info );
	type = bibtexout_type( info, "", refnum, p );
	output_type( fp, type, p->format_opts );
	if ( !( p->format_opts & BIBL_FORMAT_BIBOUT_DROPKEY ) )
		output_citekey( fp, info, refnum, p->format_opts );
	output_people( fp, info, refnum, "AUTHOR", "AUTHOR:CORP", "AUTHOR:ASIS", "author", 0,
		p->format_opts );
	output_people( fp, info, refnum, "EDITOR", "EDITOR:CORP", "EDITOR:ASIS", "editor", -1,
		p->format_opts );
	output_people( fp, info, refnum, "TRANSLATOR", "TRANSLATOR:CORP", "TRANSLATOR:ASIS", "translator", -1, p->format_opts );

	/* item=main level title */
	if ( type==TYPE_INBOOK )
		output_title( fp, info, refnum, "chapter", 0, p->format_opts );
	else
		output_title( fp, info, refnum, "title", 0, p->format_opts );

	/* item=host level title */
	if ( type==TYPE_ARTICLE )
		output_title( fp, info, refnum, "journal", 1, p->format_opts );
	else if ( type==TYPE_INBOOK ) {
		output_title( fp, info, refnum, "title", 1, p->format_opts );
		output_title( fp, info, refnum, "series", 2, p->format_opts );
	}
	else if ( type==TYPE_INPROCEEDINGS || type==TYPE_INCOLLECTION ) {
		output_title( fp, info, refnum, "booktitle", 1, p->format_opts );
		output_title( fp, info, refnum, "series", 2, p->format_opts );
	}
	else if ( type==TYPE_PHDTHESIS || type==TYPE_MASTERSTHESIS ) {
		output_title( fp, info, refnum, "series", 1, p->format_opts );
	}
	else if ( type==TYPE_BOOK || type==TYPE_COLLECTION || type==TYPE_PROCEEDINGS )
		output_title( fp, info, refnum, "series", 1, p->format_opts );

	output_date( fp, info, refnum, p->format_opts );
	output_simple( fp, info, "EDITION", "edition", p->format_opts );
	output_simple( fp, info, "PUBLISHER", "publisher", p->format_opts );
	output_simple( fp, info, "ADDRESS", "address", p->format_opts );
	output_simple( fp, info, "VOLUME", "volume", p->format_opts );
	output_issue_number( fp, info, p->format_opts );
/*	output_simple( fp, info, "ISSUE", "issue", p->format_opts );
	output_simple( fp, info, "NUMBER", "number", p->format_opts );s*/
	output_pages( fp, info, refnum, p->format_opts );
	output_simpleall( fp, info, "KEYWORD", "keywords", p->format_opts );
	output_simple( fp, info, "CONTENTS", "contents", p->format_opts );
	output_simple( fp, info, "ABSTRACT", "abstract", p->format_opts );
	output_simple( fp, info, "LOCATION", "location", p->format_opts );
	output_simple( fp, info, "DEGREEGRANTOR", "school", p->format_opts );
	output_simple( fp, info, "DEGREEGRANTOR:ASIS", "school", p->format_opts );
	output_simple( fp, info, "DEGREEGRANTOR:CORP", "school", p->format_opts );
	output_simpleall( fp, info, "NOTES", "note", p->format_opts );
	output_simpleall( fp, info, "ANNOTE", "annote", p->format_opts );
	output_simple( fp, info, "ISBN", "isbn", p->format_opts );
	output_simple( fp, info, "ISSN", "issn", p->format_opts );
	output_simple( fp, info, "DOI", "doi", p->format_opts );
	output_simpleall( fp, info, "URL", "url", p->format_opts );
	output_fileattach( fp, info, p->format_opts );
	output_arxiv( fp, info, p->format_opts );
	output_pmid( fp, info, p->format_opts );
	output_jstor( fp, info, p->format_opts );
	output_simple( fp, info, "LANGUAGE", "language", p->format_opts );
	if ( p->format_opts & BIBL_FORMAT_BIBOUT_FINALCOMMA ) fprintf( fp, "," );
	fprintf( fp, "\n}\n\n" );
	fflush( fp );
}

void
bibtexout_writeheader( FILE *outptr, param *p )
{
	if ( p->utf8bom ) utf8_writebom( outptr );
}

