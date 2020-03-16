/*
 * medin.c
 *
 * Copyright (c) Chris Putnam 2004-2019
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
#include "iso639_2.h"
#include "bibutils.h"
#include "bibformats.h"

static int medin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int medin_processf( fields *medin, const char *data, const char *filename, long nref, param *p );


/*****************************************************
 PUBLIC: void medin_initparams()
*****************************************************/
int
medin_initparams( param *pm, const char *progname )
{
	pm->readformat       = BIBL_MEDLINEIN;
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

	pm->readf    = medin_readf;
	pm->processf = medin_processf;
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
 PUBLIC: int medin_readf()
*****************************************************/

/*
 * The only difference between MEDLINE and PUBMED in format is
 * that the entire library is wrapped in <MedlineCitationSet>
 * or <PubmedArticle> tags...
 */
static char *wrapper[] = { "PubmedArticle", "MedlineCitation" };
static int nwrapper = sizeof( wrapper ) / sizeof( wrapper[0] );

static char *
medin_findstartwrapper( char *buf, int *ntype )
{
	char *startptr=NULL;
	int i;
	for ( i=0; i<nwrapper && startptr==NULL; ++i ) {
		startptr = xml_find_start( buf, wrapper[ i ] );
		if ( startptr && *ntype==-1 ) *ntype = i;
	}
	return startptr;
}

static char *
medin_findendwrapper( char *buf, int ntype )
{
	char *endptr = xml_find_end( buf, wrapper[ ntype ] );
	return endptr;
}

static int
medin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	str tmp;
	char *startptr = NULL, *endptr;
	int haveref = 0, inref = 0, file_charset = CHARSET_UNKNOWN, m, type = -1;
	str_init( &tmp );
	while ( !haveref && str_fget( fp, buf, bufsize, bufpos, line ) ) {
		if ( line->data ) {
			m = xml_getencoding( line );
			if ( m!=CHARSET_UNKNOWN ) file_charset = m;
		}
		if ( line->data ) {
			startptr = medin_findstartwrapper( line->data, &type );
		}
		if ( startptr || inref ) {
			if ( inref ) str_strcat( &tmp, line );
			else {
				str_strcatc( &tmp, startptr );
				inref = 1;
			}
			endptr = medin_findendwrapper( str_cstr( &tmp ), type );
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
 PUBLIC: int medin_processf()
*****************************************************/

typedef struct xml_convert {
	char *in;       /* The input tag */
	char *a, *aval; /* The attribute="attribute_value" pair, if nec. */
	char *out;      /* The output tag */
	int level;
} xml_convert;

static int
medin_doconvert( xml *node, fields *info, xml_convert *c, int nc, int *found )
{
	int i, fstatus;
	char *d;
	*found = 0;
	if ( !xml_has_value( node ) ) return BIBL_OK;
	d = xml_value_cstr( node );
	for ( i=0; i<nc && *found==0; ++i ) {
		if ( c[i].a==NULL ) {
			if ( xml_tag_matches( node, c[i].in ) ) {
				*found = 1;
				fstatus = fields_add( info, c[i].out, d, c[i].level );
				if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
			}
		} else {
			if ( xml_tag_has_attribute( node, c[i].in, c[i].a, c[i].aval ) ) {
				*found = 1;
				fstatus = fields_add( info, c[i].out, d, c[i].level );
				if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
			}
		}
	
	}
	return BIBL_OK;
}

/* <ArticleTitle>Mechanism and.....</ArticleTitle>
 */
static int
medin_articletitle( xml *node, fields *info )
{
	int fstatus, status = BIBL_OK;
	if ( xml_has_value( node ) ) {
		fstatus = fields_add( info, "TITLE", xml_value_cstr( node ), 0 );
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
	}
	return status;
}

/*            <MedlineDate>2003 Jan-Feb</MedlineDate> */
static int
medin_medlinedate( fields *info, const char *p, int level )
{
	int fstatus;
	str tmp;

	str_init( &tmp );

	p = str_cpytodelim( &tmp, skip_ws( p ), " \t\n\r", 0 );
	if ( str_memerr( &tmp ) ) return BIBL_ERR_MEMERR;

	if ( str_has_value( &tmp ) ) {
		fstatus = fields_add( info, "PARTDATE:YEAR", str_cstr( &tmp ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	p = str_cpytodelim( &tmp, skip_ws( p ), " \t\n\r", 0 );
	if ( str_memerr( &tmp ) ) return BIBL_ERR_MEMERR;

	if ( str_has_value( &tmp ) ) {
		str_findreplace( &tmp, "-", "/" );
		fstatus = fields_add( info, "PARTDATE:MONTH", str_cstr( &tmp ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	(void) str_cpytodelim( &tmp, skip_ws( p ), " \t\n\r", 0 );
	if ( str_memerr( &tmp ) ) return BIBL_ERR_MEMERR;

	if ( str_has_value( &tmp ) ) {
		fstatus = fields_add( info, "PARTDATE:DAY", str_cstr( &tmp ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	str_free( &tmp );

	return BIBL_OK;
}

/* <Langauge>eng</Language>
 */
static int
medin_language( xml *node, fields *info, int level )
{
	char *code, *language;
	int fstatus;
	code = xml_value_cstr( node );
	if ( !code ) return BIBL_OK;
	language = iso639_2_from_code( code );
	if ( language )
		fstatus = fields_add( info, "LANGUAGE", language, level );
	else
		fstatus = fields_add( info, "LANGUAGE", code, level );
	if ( fstatus==FIELDS_OK ) return BIBL_OK;
	else return BIBL_ERR_MEMERR;
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
medin_journal1( xml *node, fields *info )
{
	xml_convert c[] = {
		{ "Title",           NULL, NULL, "TITLE",          1 },
		{ "ISOAbbreviation", NULL, NULL, "SHORTTITLE",     1 },
		{ "ISSN",            NULL, NULL, "ISSN",           1 },
		{ "Volume",          NULL, NULL, "VOLUME",         1 },
		{ "Issue",           NULL, NULL, "ISSUE",          1 },
		{ "Year",            NULL, NULL, "PARTDATE:YEAR",  1 },
		{ "Month",           NULL, NULL, "PARTDATE:MONTH", 1 },
		{ "Day",             NULL, NULL, "PARTDATE:DAY",   1 },
	};
	int nc = sizeof( c ) / sizeof( c[0] ), status, found;
	if ( xml_has_value( node ) ) {
		status = medin_doconvert( node, info, c, nc, &found );
		if ( status!=BIBL_OK ) return status;
		if ( !found ) {
			if ( xml_tag_matches( node, "MedlineDate" ) ) {
				status = medin_medlinedate( info, xml_value_cstr( node ), 1 );
				if ( status!=BIBL_OK ) return status;
			}
			if ( xml_tag_matches( node, "Language" ) ) {
				status = medin_language( node, info, 1 );
				if ( status!=BIBL_OK ) return status;
			}
		}
	}
	if ( node->down ) {
		status = medin_journal1( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = medin_journal1( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/* <Pagination>
 *    <MedlinePgn>12111-6</MedlinePgn>
 * </Pagination>
 */
static int
medin_pagination( xml *node, fields *info )
{
	int i, fstatus, status;
	str sp, ep;
	const char *p, *pp;
	if ( xml_tag_matches( node, "MedlinePgn" ) && node->value.len ) {
		strs_init( &sp, &ep, NULL );
		p = str_cpytodelim( &sp, xml_value_cstr( node ), "-", 1 );
		if ( str_memerr( &sp ) ) return BIBL_ERR_MEMERR;
		if ( str_has_value( &sp ) ) {
			fstatus = fields_add( info, "PAGES:START", str_cstr( &sp ), 1 );
			if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
		(void) str_cpytodelim( &ep, p, "", 0 );
		if ( str_memerr( &ep ) ) return BIBL_ERR_MEMERR;
		if ( str_has_value( &ep ) ) {
			if ( sp.len > ep.len ) {
				for ( i=sp.len-ep.len; i<sp.len; ++i )
					sp.data[i] = ep.data[i-sp.len+ep.len];
				pp = sp.data;
			} else  pp = ep.data;
			fstatus = fields_add( info, "PAGES:STOP", pp, 1 );
			if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
		strs_free( &sp, &ep, NULL );
	}
	if ( node->down ) {
		status = medin_pagination( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) {
		status = medin_pagination( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

/* <Abstract>
 *    <AbstractText>ljwejrelr</AbstractText>
 * </Abstract>
 */
static int
medin_abstract( xml *node, fields *info )
{
	int fstatus;
	if ( xml_tag_matches_has_value( node, "AbstractText" ) ) {
		fstatus = fields_add( info, "ABSTRACT", xml_value_cstr( node ), 0 );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	} else if ( node->next ) return medin_abstract( node->next, info );
	return BIBL_OK;
}

/* <AuthorList CompleteYN="Y">
 *    <Author>
 *        <LastName>Barondeau</LastName>
 *        <ForeName>David P</ForeName>
 *        ( or <FirstName>David P</FirstName> )
 *        <Initials>DP</Initials>
 *    </Author>
 *    <Author>
 *        <CollectiveName>Organization</CollectiveName>
 *    </Author>
 * </AuthorList>
 */
static int
medin_author( xml *node, str *name )
{
	char *p;
	if ( xml_tag_matches( node, "LastName" ) ) {
		if ( str_has_value( name ) ) {
			str_prepend( name, "|" );
			str_prepend( name, xml_value_cstr( node ) );
		}
		else str_strcat( name, xml_value( node ) );
	} else if ( xml_tag_matches( node, "ForeName" ) ||
	            xml_tag_matches( node, "FirstName" ) ) {
		p = xml_value_cstr( node );
		while ( p && *p ) {
			if ( str_has_value( name ) ) str_addchar( name, '|' );
			while ( *p==' ' ) p++;
			while ( *p && *p!=' ' ) str_addchar( name, *p++ );
		}
	} else if ( xml_tag_matches( node, "Initials" ) && !strchr( name->data, '|' )) {
		p = xml_value_cstr( node );
		while ( p && *p ) {
			if ( str_has_value( name ) ) str_addchar( name, '|' );
			if ( !is_ws(*p) ) str_addchar( name, *p++ );
		}
	}
	if ( node->next ) medin_author( node->next, name );
	return BIBL_OK;
}

static int
medin_corpauthor( xml *node, str *name )
{
	if ( xml_tag_matches( node, "CollectiveName" ) ) {
		str_strcpy( name, xml_value( node ) );
	} else if ( node->next ) medin_corpauthor( node->next, name );
	return BIBL_OK;
}

static int
medin_authorlist( xml *node, fields *info )
{
	int fstatus, status;
	str name;
	char *tag;
	str_init( &name );
	node = node->down;
	while ( node ) {
		if ( xml_tag_matches( node, "Author" ) && node->down ) {
			status = medin_author( node->down, &name );
			tag = "AUTHOR";
			if ( str_is_empty( &name ) ) {
				status = medin_corpauthor( node->down, &name );
				tag = "AUTHOR:CORP";
			}
			if ( str_memerr( &name ) || status!=BIBL_OK ) return BIBL_ERR_MEMERR;
			if ( str_has_value( &name ) ) {
				fstatus = fields_add(info,tag,name.data,0);
				if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
			}
			str_empty( &name );
		}
		node = node->next;
	}
	str_free( &name );
	return BIBL_OK;
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
medin_journal2( xml *node, fields *info )
{
	int fstatus, status = BIBL_OK;
	if ( xml_tag_matches_has_value( node, "MedlineTA" ) && fields_find( info, "TITLE", LEVEL_HOST )==FIELDS_NOTFOUND ) {
		fstatus = fields_add( info, "TITLE", xml_value_cstr( node ), 1 );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = medin_journal2( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = medin_journal2( node->next, info );
	return status;
}

/*
<MeshHeadingList>
<MeshHeading>
<DescriptorName MajorTopicYN="N">Biophysics</DescriptorName>
</MeshHeading>
<MeshHeading>
<DescriptorName MajorTopicYN="N">Crystallography, X-Ray</DescriptorName>
</MeshHeading>
</MeshHeadingList>
*/
static int
medin_meshheading( xml *node, fields *info )
{
	int fstatus, status = BIBL_OK;
	if ( xml_tag_matches_has_value( node, "DescriptorName" ) ) {
		fstatus = fields_add( info, "KEYWORD", xml_value_cstr( node ), 0 );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->next ) status = medin_meshheading( node->next, info );
	return status;
}

static int
medin_meshheadinglist( xml *node, fields *info )
{
	int status = BIBL_OK;
	if ( xml_tag_matches( node, "MeshHeading" ) && node->down ) {
		status = medin_meshheading( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = medin_meshheadinglist( node->next, info );
	return status;
}

/* <PubmedData>
 *     ....
 *     <ArticleIdList>
 *         <ArticleId IdType="pubmed">14523232</ArticleId>
 *         <ArticleId IdType="doi">10.1073/pnas.2133463100</ArticleId>
 *         <ArticleId IdType="pii">2133463100</ArticleId>
 *         <ArticleId IdType="pmc">PMC4833866</ArticleId>
 *     </ArticleIdList>
 * </PubmedData>
 *
 * I think "pii" is "Publisher Item Identifier"
 */
static int
medin_pubmeddata( xml *node, fields *info )
{
	xml_convert c[] = {
		{ "ArticleId", "IdType", "doi",     "DOI",     0 },
		{ "ArticleId", "IdType", "pubmed",  "PMID",    0 },
		{ "ArticleId", "IdType", "medline", "MEDLINE", 0 },
		{ "ArticleId", "IdType", "pmc",     "PMC",     0 },
		{ "ArticleId", "IdType", "pii",     "PII",     0 },
	};
	int nc = sizeof( c ) / sizeof( c[0] ), found, status;
	status = medin_doconvert( node, info, c, nc, &found );
	if ( status!=BIBL_OK ) return status;
	if ( node->next ) {
		status = medin_pubmeddata( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->down ) {
		status = medin_pubmeddata( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	return BIBL_OK;
}

static int
medin_article( xml *node, fields *info )
{
	int fstatus, status = BIBL_OK;
	if ( xml_tag_matches( node, "Journal" ) )
		status = medin_journal1( node, info );
	else if ( xml_tag_matches( node, "ArticleTitle" ) )
		status = medin_articletitle( node, info );
	else if ( xml_tag_matches( node, "Pagination" ) && node->down )
		status = medin_pagination( node->down, info );
	else if ( xml_tag_matches( node, "Abstract" ) && node->down )
		status = medin_abstract( node->down, info );
	else if ( xml_tag_matches( node, "AuthorList" ) )
		status = medin_authorlist( node, info );
	else if ( xml_tag_matches( node, "Language" ) )
		status = medin_language( node, info, 0 );
	else if ( xml_tag_matches( node, "Affiliation" ) ) {
		fstatus = fields_add( info, "ADDRESS", xml_value_cstr( node ), 0 );
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
	}
	if ( status!=BIBL_OK ) return status;
	if ( node->next ) status = medin_article( node->next, info );
	return status;
}

static int
medin_medlinecitation( xml *node, fields *info )
{
	int fstatus, status = BIBL_OK;
	if ( xml_tag_matches_has_value( node, "PMID" ) ) {
		fstatus = fields_add( info, "PMID", xml_value_cstr( node ), 0 );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		if ( xml_tag_matches( node, "Article" ) ) {
			status = medin_article( node->down, info );
		} else if ( xml_tag_matches( node, "MedlineJournalInfo" ) ) {
			status = medin_journal2( node->down, info );
		} else if ( xml_tag_matches( node, "MeshHeadingList" ) )
			status = medin_meshheadinglist( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = medin_medlinecitation( node->next, info );
	return status;
}

static int
medin_pubmedarticle( xml *node, fields *info )
{
	int status = BIBL_OK;
	if ( node->down ) {
		if ( xml_tag_matches( node, "MedlineCitation" ) )
			status = medin_medlinecitation( node->down, info );
		else if ( xml_tag_matches( node, "PubmedData" ) )
			status = medin_pubmeddata( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = medin_pubmedarticle( node->next, info );
	return status;
}

static int
medin_assembleref( xml *node, fields *info )
{
	int status = BIBL_OK;
	if ( node->down ) {
		if ( xml_tag_matches( node, "PubmedArticle" ) )
			status = medin_pubmedarticle( node->down, info );
		else if ( xml_tag_matches( node, "MedlineCitation" ) )
			status = medin_medlinecitation( node->down, info );
		else
			status = medin_assembleref( node->down, info );
	}
	if ( status!=BIBL_OK ) return status;

	if ( node->next ) {
		status = medin_assembleref( node->next, info );
		if ( status!=BIBL_OK ) return status;
	}

	/* assume everything is a journal article */
	if ( fields_num( info ) ) {
		status = fields_add( info, "RESOURCE", "text", 0 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		status = fields_add( info, "ISSUANCE", "continuing", 1 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		status = fields_add( info, "GENRE:MARC", "periodical", 1 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		status = fields_add( info, "GENRE:BIBUTILS", "academic journal", 1 );
		if ( status!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		status = BIBL_OK;
	}

	return status;
}

static int
medin_processf( fields *medin, const char *data, const char *filename, long nref, param *p )
{
	int status;
	xml top;

	xml_init( &top );
	xml_parse( data, &top );
	status = medin_assembleref( &top, medin );
	xml_free( &top );

	if ( status==BIBL_OK ) return 1;
	return 0;
}
