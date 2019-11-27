/*
 * ebiin.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Program and source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include "is_ws.h"
#include "str.h"
#include "str_conv.h"
#include "fields.h"
#include "bu_auth.h"
#include "marc_auth.h"
#include "xml.h"
#include "xml_encoding.h"
#include "bibformats.h"

static int ebiin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int ebiin_processf( fields *ebiin, const char *data, const char *filename, long nref, param *p );


/*****************************************************
 PUBLIC: void ebiin_initparams()
*****************************************************/
int
ebiin_initparams( param *pm, const char *progname )
{
	pm->readformat       = BIBL_EBIIN;
	pm->charsetin        = BIBL_CHARSET_UNICODE;
	pm->charsetin_src    = BIBL_SRC_DEFAULT;
	pm->latexin          = 0;
	pm->xmlin            = 1;
	pm->utf8in           = 1;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->output_raw       = BIBL_RAW_WITHMAKEREFID |
	                       BIBL_RAW_WITHCHARCONVERT;

	pm->readf    = ebiin_readf;
	pm->processf = ebiin_processf;
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
 PUBLIC: int ebiin_readf()
*****************************************************/
static int
ebiin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	int haveref = 0, inref = 0, file_charset = CHARSET_UNKNOWN, m;
	char *startptr = NULL, *endptr;
	str tmp;
	str_init( &tmp );
	while ( !haveref && str_fget( fp, buf, bufsize, bufpos, line ) ) {
		if ( line->data ) {
			m = xml_getencoding( line );
			if ( m!=CHARSET_UNKNOWN ) file_charset = m;
		}
		if ( line->data )
			startptr = xml_find_start( line->data, "Publication" );
		if ( startptr || inref ) {
			if ( inref ) str_strcat( &tmp, line );
			else {
				str_strcatc( &tmp, startptr );
				inref = 1;
			}
			endptr = xml_find_end( str_cstr( &tmp ), "Publication" );
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
 PUBLIC: int ebiin_processf()
*****************************************************/

typedef struct xml_convert {
	char *in;       /* The input tag */
	char *a, *aval; /* The attribute="attribute_value" pair, if nec. */
	char *out;      /* The output tag */
	int level;
} xml_convert;

static int
ebiin_doconvert( xml *node, fields *info, xml_convert *c, int nc, int *found )
{
	int i, status;
	char *d;

	if ( !xml_has_value( node ) ) goto out;

	d = xml_value_cstr( node );
	for ( i=0; i<nc; ++i ) {
		if ( c[i].a==NULL ) {
			if ( xml_tag_matches( node, c[i].in ) ) {
				*found = 1;
				status = fields_add( info, c[i].out, d, c[i].level );
				if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
				else return BIBL_OK;
			}
		} else {
			if ( xml_tag_has_attribute( node, c[i].in, c[i].a, c[i].aval ) ){
				*found = 1;
				status = fields_add( info, c[i].out, d, c[i].level );
				if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
				else return BIBL_OK;
			}
		}
	
	}
out:
	*found = 0;
	return BIBL_OK;
}

/* <ArticleTitle>Mechanism and.....</ArticleTitle>
 * and
 * <Title>Mechanism and....</Title>
 */
static int
ebiin_title( xml *node, fields *info, int title_level )
{
	int status;
	if ( xml_has_value( node ) ) {
		status = fields_add( info, "TITLE", xml_value_cstr( node ), title_level );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	return BIBL_OK;
}

/* ebiin_medlinedate()
 *
 *   - extract medline information from entries like:
 *             <MedlineDate>2003 Jan-Feb</MedlineDate>
 */
static int
ebiin_medlinedate_year( fields *info, const char *p, int level, const char **end )
{
	int fstatus, status = BIBL_OK;
	str s;

	str_init( &s );

	*end = str_cpytodelim( &s, p, " \t\n\r", 0 );
	if ( str_memerr( &s ) ) {
		status = BIBL_ERR_MEMERR;
		goto out;
	}
	if ( str_has_value( &s ) ) {
		fstatus = fields_add( info, "PARTDATE:YEAR", str_cstr( &s ), level );
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
	}
out:
	str_free( &s );
	return status;
}
static int
ebiin_medlinedate_month( fields *info, const char *p, int level, const char **end )
{
	int fstatus, status = BIBL_OK;
	str s;

	str_init( &s );

	*end = str_cpytodelim( &s, p, " \t\n\r", 0 );
	str_findreplace( &s, "-", "/" );
	if ( str_memerr( &s ) ) {
		status = BIBL_ERR_MEMERR;
		goto out;
	}
	if ( str_has_value( &s ) ) {
		fstatus = fields_add( info, "PARTDATE:MONTH", str_cstr( &s ), level );
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
	}
out:
	str_free( &s );
	return status;
}

static int
ebiin_medlinedate_day( fields *info, const char *p, int level, const char **end )
{
	int fstatus, status = BIBL_OK;
	str s;

	str_init( &s );

	*end = str_cpytodelim( &s, p, " \t\n\r", 0 );
	if ( str_memerr( &s ) ) {
		status = BIBL_ERR_MEMERR;
		goto out;
	}
	if ( str_has_value( &s ) ) {
		fstatus = fields_add( info, "PARTDATE:DAY", str_cstr( &s ), level );
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
	}
out:
	str_free( &s );
	return status;
}

static int
ebiin_medlinedate( fields *info, xml *node, int level )
{
	int status = BIBL_OK;
	const char *p;

	if ( !xml_has_value( node ) ) return status;

	p = xml_value_cstr( node );

	if ( *p )
		status = ebiin_medlinedate_year( info, skip_ws( p ), level, &p );
	if ( *p && status==BIBL_OK )
		status = ebiin_medlinedate_month( info, skip_ws( p ), level, &p );
	if ( *p && status==BIBL_OK )
		status = ebiin_medlinedate_day( info, skip_ws( p ), level, &p );

	return status;
}

/* <Journal>
 *    <ISSN>0027-8424</ISSN>
 *    <JournalIssue PrintYN="Y">
 *       <Volume>100</Volume>
 *       <Issue>21</Issue>
 *       <PubDate>
 *          <Year>2003</Year>
 *          <Month>Oct</Month>
 *          <Day>14</Day>
 *       </PubDate>
 *    </Journal Issue>
 * </Journal>
 *
 * or....
 *
 * <Journal>
 *    <ISSN IssnType="Print">0735-0414</ISSN>
 *    <JournalIssue CitedMedium="Print">
 *        <Volume>38</Volume>
 *        <Issue>1</Issue>
 *        <PubDate>
 *            <MedlineDate>2003 Jan-Feb</MedlineDate>
 *        </PubDate>
 *    </JournalIssue>
 *    <Title>Alcohol and alcoholism (Oxford, Oxfordshire)  </Title>
 *    <ISOAbbreviation>Alcohol Alcohol.</ISOAbbreviation>
 * </Journal>
 */
static int
ebiin_journal1( xml *node, fields *info )
{
	xml_convert c[] = {
		{ "ISSN",     NULL, NULL, "ISSN",           1 },
		{ "Volume",   NULL, NULL, "VOLUME",         1 },
		{ "Issue",    NULL, NULL, "ISSUE",          1 },
		{ "Year",     NULL, NULL, "PARTDATE:YEAR",  1 },
		{ "Month",    NULL, NULL, "PARTDATE:MONTH", 1 },
		{ "Day",      NULL, NULL, "PARTDATE:DAY",   1 },
		{ "Language", NULL, NULL, "LANGUAGE",       1 },
	};
	int nc = sizeof( c ) / sizeof( c[0] ), status, found;
	if ( xml_has_value( node ) ) {
		status = ebiin_doconvert( node, info, c, nc, &found );
		if ( status!=BIBL_OK ) return status;
		if ( !found ) {
			if ( xml_tag_matches( node, "MedlineDate" ) ) {
				status = ebiin_medlinedate( info, node, LEVEL_HOST );
				if ( status!=BIBL_OK ) return status;
			}
		}
	}
	if ( node->down ) {
		status = ebiin_journal1( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_journal1( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/* <Pagination>
 *    <MedlinePgn>12111-6</MedlinePgn>
 * </Pagination>
 */
static int
ebiin_pages( fields *info, const char *p )
{
	int i, status, ret = BIBL_OK;
	const int level = 1;
	str sp, ep, *up;

	strs_init( &sp, &ep, NULL );

	/* ...start page */
	p = str_cpytodelim( &sp, skip_ws( p ), "-", 1 );
	if ( str_memerr( &sp ) ) {
		ret = BIBL_ERR_MEMERR;
		goto out;
	}

	/* ...end page */
	(void) str_cpytodelim( &ep, skip_ws( p ), " \t\n\r", 0 );
	if ( str_memerr( &ep ) ) {
		ret = BIBL_ERR_MEMERR;
		goto out;
	}

	if ( sp.len ) {
		status = fields_add( info, "PAGES:START", sp.data, level );
		if ( status!=FIELDS_OK ) {
			ret = BIBL_ERR_MEMERR;
			goto out;
		}
	}
	if ( ep.len ) {
		if ( sp.len > ep.len ) {
			for ( i=sp.len-ep.len; i<sp.len; ++i )
				sp.data[i] = ep.data[i-sp.len+ep.len];
			up = &(sp);
		} else up = &(ep);
		status = fields_add( info, "PAGES:STOP", up->data, level );
		if ( status!=FIELDS_OK ) ret = BIBL_ERR_MEMERR;
	}

out:
	strs_free( &sp, &ep, NULL );
	return ret;
}
static int
ebiin_pagination( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches_has_value( node, "Pages" ) ) {
		status = ebiin_pages( info, xml_value_cstr( node ) );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->down ) {
		status = ebiin_pagination( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_pagination( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/* <Abstract>
 *    <AbstractText>ljwejrelr</AbstractText>
 * </Abstract>
 */
static int
ebiin_abstract( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches_has_value( node, "AbstractText" ) ) {
		status = fields_add( info, "ABSTRACT", xml_value_cstr( node ), 0 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	else if ( node->next ) {
		status = ebiin_abstract( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/* <AuthorList CompleteYN="Y">
 *    <Author>
 *        <LastName>Barondeau</LastName>
 *        <ForeName>David P</ForeName>
 *        ( or <FirstName>David P</FirstName> )
 *        <Initials>DP</Initials>
 *    </Author>
 * </AuthorList>
 */
static int
ebiin_author( xml *node, str *name )
{
	int status;
	char *p;
	if ( xml_tag_matches( node, "LastName" ) ) {
		if ( name->len ) {
			str_prepend( name, "|" );
			str_prepend( name, xml_value_cstr( node ) );
		}
		else str_strcat( name, xml_value( node ) );
	} else if ( xml_tag_matches( node, "ForeName" ) ||
	            xml_tag_matches( node, "FirstName" ) ) {
		p = xml_value_cstr( node );
		while ( p && *p ) {
			if ( name->len ) str_addchar( name, '|' );
			while ( *p==' ' ) p++;
			while ( *p && *p!=' ' ) str_addchar( name, *p++ );
		}
	} else if ( xml_tag_matches( node, "Initials" ) && !strchr( name->data, '|' ) ) {
		p = xml_value_cstr( node );
		while ( p && *p ) {
			if ( name->len ) str_addchar( name, '|' );
			if ( !is_ws(*p ) ) str_addchar( name, *p++ );
		}
	}
	if ( str_memerr( name ) ) return BIBL_ERR_MEMERR;
		 
	if ( node->down ) {
		status = ebiin_author( node->down, name );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_author( node->next, name );
		if ( status!=BIBL_OK ) return status;
	}

	return BIBL_OK;
}
static int
ebiin_authorlist( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	str name;

	str_init( &name );
	node = node->down;
	while ( node ) {
		if ( xml_tag_matches( node, "Author" ) && node->down ) {
			status = ebiin_author( node->down, &name );
			if ( status!=BIBL_OK ) goto out;
			if ( name.len ) {
				fstatus = fields_add(info,"AUTHOR",name.data,level);
				if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
				str_empty( &name );
			}
		}
		node = node->next;
	}
out:
	str_free( &name );
	return status;
}

/* <PublicationTypeList>
 *    <PublicationType>Journal Article</PublicationType>
 * </PublicationTypeList>
 */

/* <MedlineJournalInfo>
 *    <Country>United States</Country>
 *    <MedlineTA>Proc Natl Acad Sci U S A</MedlineTA>
 *    <NlmUniqueID>7507876</NlmUniqueID>
 * </MedlineJournalInfo>
 */

static int
ebiin_journal2( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches_has_value( node, "TitleAbbreviation" ) ) {
		status = fields_add( info, "TITLE", xml_value_cstr( node ), 1 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = ebiin_journal2( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_journal2( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/*
 * <MeshHeadingList>
 *   <MeshHeading>
 *     <DescriptorName MajorTopicYN="N">Biophysics</DescriptorName>
 *   </MeshHeading>
 *   <MeshHeading>
 *     <DescriptorName MajorTopicYN="N">Crystallography, X-Ray</DescriptorName>
 *   </MeshHeading>
 * </MeshHeadingList>
*/
static int
ebiin_meshheading( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches_has_value( node, "DescriptorName" ) ) {
		status = fields_add( info, "KEYWORD", xml_value_cstr( node ), 0 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->next ) {
		status = ebiin_meshheading( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
ebiin_meshheadinglist( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches( node, "MeshHeading" ) && node->down ) {
		status = ebiin_meshheading( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_meshheadinglist( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
ebiin_book( xml *node, fields *info, int book_level )
{
	xml_convert book[] = {
		{ "Publisher",              NULL, NULL, "PUBLISHER",      0 },
		{ "Language",               NULL, NULL, "LANGUAGE",       0 },
		{ "ISBN10",                 NULL, NULL, "ISBN",           0 },
		{ "ISBN13",                 NULL, NULL, "ISBN13",         0 },
		{ "Year",                   NULL, NULL, "DATE:YEAR",      0 },
		{ "Month",                  NULL, NULL, "DATE:MONTH",     0 },
		{ "Day",                    NULL, NULL, "DATE:DAY",       0 },
		{ "PageTotal",              NULL, NULL, "PAGES:TOTAL",    0 },
		{ "SeriesName",             NULL, NULL, "TITLE",          1 },
		{ "SeriesISSN",             NULL, NULL, "ISSN",           0 },
		{ "OtherReportInformation", NULL, NULL, "NOTES",          0 },
		{ "Edition",                NULL, NULL, "EDITION",        0 },
	};
	int nbook = sizeof( book ) / sizeof( book[0] );
	xml_convert inbook[] = {
		{ "Publisher",              NULL, NULL, "PUBLISHER",      1 },
		{ "Language",               NULL, NULL, "LANGUAGE",       0 },
		{ "ISBN10",                 NULL, NULL, "ISBN",           1 },
		{ "ISBN13",                 NULL, NULL, "ISBN13",         1 },
		{ "Year",                   NULL, NULL, "PARTDATE:YEAR",  1 },
		{ "Month",                  NULL, NULL, "PARTDATE:MONTH", 1 },
		{ "Day",                    NULL, NULL, "PARTDATE:DAY",   1 },
		{ "PageTotal",              NULL, NULL, "PAGES:TOTAL",    1 },
		{ "SeriesName",             NULL, NULL, "TITLE",          2 },
		{ "SeriesISSN",             NULL, NULL, "ISSN",           1 },
		{ "OtherReportInformation", NULL, NULL, "NOTES",          1 },
		{ "Edition",                NULL, NULL, "EDITION",        1 },
	};
	int ninbook = sizeof( inbook ) / sizeof( inbook[0] );
	int nc, status, found;
	xml_convert *c;

	if ( book_level==0 ) {
		c  = book;
		nc = nbook;
	}
	else {
		c  = inbook;
		nc = ninbook;
	}

	status = ebiin_doconvert( node, info, c, nc, &found );
	if ( status!=BIBL_OK ) return status;

	if ( !found ) {
		status = BIBL_OK;
		if ( xml_tag_matches( node, "MedlineDate" ) )
			status = ebiin_medlinedate( info, node, book_level );
		else if ( xml_tag_matches( node, "Title" ) )
			status = ebiin_title( node, info, book_level );
		else if ( xml_tag_matches( node, "Pagination" ) && node->down )
			status = ebiin_pagination( node->down, info );
		else if ( xml_tag_matches( node, "Abstract" ) && node->down )
			status = ebiin_abstract( node->down, info );
		else if ( xml_tag_matches( node, "AuthorList" ) )
			status = ebiin_authorlist( node, info, book_level );
		else if ( xml_tag_matches( node, "PubDate" ) && node->down )
			status = ebiin_book( node->down, info, book_level );
		if ( status!=BIBL_OK ) return status;
	}

	if ( node->next ) {
		status = ebiin_book( node->next, info, book_level );
		if ( status!=BIBL_OK ) return status;
	}

	return BIBL_OK;
}

static int
ebiin_article( xml *node, fields *info )
{
	int status = BIBL_OK;

	if ( xml_tag_matches( node, "Journal" ) )
		status = ebiin_journal1( node, info );
	else if ( node->down && ( xml_tag_matches( node, "Book" ) ||
			xml_tag_matches(node, "Report") ))
		status = ebiin_book( node->down, info, 1 );
	else if ( xml_tag_matches( node, "ArticleTitle" ) )
		status = ebiin_title( node, info, 0 );
	else if ( xml_tag_matches( node, "Pagination" ) && node->down )
		status = ebiin_pagination( node->down, info );
	else if ( xml_tag_matches( node, "Abstract" ) && node->down )
		status = ebiin_abstract( node->down, info );
	else if ( xml_tag_matches( node, "AuthorList" ) )
		status = ebiin_authorlist( node, info, 0 );
	if ( status!=BIBL_OK ) return status;

	if ( node->next ) {
		status = ebiin_article( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}

	return BIBL_OK;
}

static int
ebiin_publication( xml *node, fields *info )
{
	int status = BIBL_OK;
	if ( node->down ) {
		if ( xml_tag_matches( node, "Article" ) )
			status = ebiin_article( node->down, info );
		else if ( xml_tag_matches( node, "Book" ) )
			status = ebiin_book( node->down, info, 0 );
		else if ( xml_tag_matches( node, "Report" ) )
			status = ebiin_book( node->down, info, 0 );
		else if ( xml_tag_matches( node, "JournalInfo" ) )
			status = ebiin_journal2( node->down, info );
		else if ( xml_tag_matches( node, "MeshHeadingList" ) )
			status = ebiin_meshheadinglist( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_publication( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/* Call with the "Publication" node */
static int
ebiin_fixtype( xml *node, fields *info )
{
	char *resource = NULL, *issuance = NULL, *genre1 = NULL, *genre2 = NULL;
	str *type;
	int reslvl, isslvl, gen1lvl, gen2lvl;
	int status;

	type = xml_attribute( node, "Type" );
	if ( !type || type->len==0 ) return BIBL_OK;

	if ( !strcmp( type->data, "JournalArticle" ) ) {
		resource = "text";
		issuance = "continuing";
		genre1   = "periodical";
		genre2   = "academic journal";
		reslvl   = 0;
		isslvl   = 1;
		gen1lvl  = 1;
		gen2lvl  = 1;
	} else if ( !strcmp( type->data, "Book" ) ) {
		resource = "text";
		issuance = "monographic";
		genre1   = "book";
		reslvl   = 0;
		isslvl   = 0;
		gen1lvl  = 0;
	} else if ( !strcmp( type->data, "BookArticle" ) ) {
		resource = "text";
		issuance = "monographic";
		genre1   = "book";
		reslvl   = 0;
		isslvl   = 1;
		gen1lvl  = 1;
	}

	if ( resource ) {
		status = fields_add( info, "RESOURCE", resource, reslvl );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( issuance ) {
		status = fields_add( info, "ISSUANCE", issuance, isslvl );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( genre1 ) {
		if ( is_marc_genre( genre1 ) )
			status = fields_add( info, "GENRE:MARC", genre1, gen1lvl );
		else if ( is_bu_genre( genre1 ) )
			status = fields_add( info, "GENRE:BIBUTILS", genre1, gen1lvl );
		else
			status = fields_add( info, "GENRE:UNKNOWN", genre1, gen1lvl );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( genre2 ) {
		if ( is_marc_genre( genre2 ) )
			status = fields_add( info, "GENRE:MARC", genre2, gen2lvl );
		else if ( is_bu_genre( genre2 ) )
			status = fields_add( info, "GENRE:BIBUTILS", genre2, gen2lvl );
		else
			status = fields_add( info, "GENRE:UNKNOWN", genre2, gen2lvl );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	return BIBL_OK;
}

static int
ebiin_assembleref( xml *node, fields *info )
{
	int status;
	if ( xml_tag_matches( node, "Publication" ) && node->down ) {
		status = ebiin_fixtype( node, info );
		if ( status!=BIBL_OK ) return status;
		status = ebiin_publication( node->down, info );
		if ( status!=BIBL_OK ) return status;
	} else if ( node->down ) {
		status = ebiin_assembleref( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = ebiin_assembleref( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
ebiin_processf( fields *ebiin, const char *data, const char *filename, long nref, param *p )
{
	int status;
	xml top;

	xml_init( &top );
	xml_parse( data, &top );
	status = ebiin_assembleref( &top, ebiin );
	xml_free( &top );

	return ( status==BIBL_OK ) ? 1 : 0;
}
