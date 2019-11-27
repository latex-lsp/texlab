/*
 * url.c
 *
 * doi_to_url()
 * Handle outputing DOI as a URL (Endnote and RIS formats)
 *     1) Append https://doi.org as necessary
 *     2) Check for overlap with pre-existing URL for the DOI
 *
 * is_doi()
 * Check for DOI buried in another field.
 *
 * Copyright (c) Chris Putnam 2008-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "bibutils.h"
#include "url.h"

static void
construct_url( char *prefix, str *id, str *id_url, char sep )
{
	if ( !strncasecmp( str_cstr( id ), "http:", 5 ) )
		str_strcpy( id_url, id );
	else {
		str_strcpyc( id_url, prefix );
		if ( sep!='\0' ) {
			if ( id->data[0]!=sep ) str_addchar( id_url, sep );
		}
		str_strcat( id_url, id );
	}
}

static int
url_exists( fields *f, char *urltag, str *doi_url )
{
	int i, n;
	if ( urltag ) {
		n = fields_num( f );
		for ( i=0; i<n; ++i ) {
			if ( strcmp( fields_tag( f, i, FIELDS_CHRP ), urltag ) ) continue;
			if ( strcmp( fields_value( f, i, FIELDS_CHRP ), str_cstr( doi_url ) ) ) continue;
			return 1;
		}
	}
	return 0;
}

static void
xxx_to_url( fields *f, int n, char *http_prefix, char *urltag, str *xxx_url, char sep )
{
	str_empty( xxx_url );
	construct_url( http_prefix, fields_value( f, n, FIELDS_STRP ), xxx_url, sep );
	if ( url_exists( f, urltag, xxx_url ) )
		str_empty( xxx_url );
}
void
doi_to_url( fields *f, int n, char *urltag, str *url )
{
	xxx_to_url( f, n, "https://doi.org", urltag, url, '/' );
}
void
jstor_to_url( fields *f, int n, char *urltag, str *url )
{
	xxx_to_url( f, n, "http://www.jstor.org/stable", urltag, url, '/' );
}
void
pmid_to_url( fields *f, int n, char *urltag, str *url )
{
	xxx_to_url( f, n, "http://www.ncbi.nlm.nih.gov/pubmed", urltag, url, '/' );
}
void
pmc_to_url( fields *f, int n, char *urltag, str *url )
{
	xxx_to_url( f, n, "http://www.ncbi.nlm.nih.gov/pmc/articles", urltag, url, '/' );
}
void
arxiv_to_url( fields *f, int n, char *urltag, str *url )
{
	xxx_to_url( f, n, "http://arxiv.org/abs", urltag, url, '/' );
}
void
mrnumber_to_url( fields *f, int n, char *urltag, str *url )
{
	xxx_to_url( f, n, "http://www.ams.org/mathscinet-getitem?mr=", urltag, url, '\0' );
}

/* Rules for the pattern:
 *   '#' = number
 *   isalpha() = match precisely (matchcase==1) or match regardless of case
 *   	(matchcase==0)
 *   all others must match precisely
 */
static int
string_pattern( char *s, char *pattern, int matchcase )
{
	int patlen, match, i;
	patlen = strlen( pattern );
	if ( strlen( s ) < patlen ) return 0; /* too short */
	for ( i=0; i<patlen; ++i ) {
		match = 0;
		if ( pattern[i]=='#' ) {
			if ( isdigit( (unsigned char)s[i] ) ) match = 1;
		} else if ( !matchcase && isalpha( (unsigned char)pattern[i] ) ) {
			if ( tolower((unsigned char)pattern[i])==tolower((unsigned char)s[i])) match = 1;
		} else {
			if ( pattern[i] == s[i] ) match = 1;
		}
		if ( !match ) return 0;
	}
	return 1;
}

/* science direct is now doing "M3  - doi: DOI: 10.xxxx/xxxxx" */
/* elsevier is doing "DO - https://doi.org/xx.xxxx/xxxx..." */
int
is_doi( char *s )
{
	if ( string_pattern( s, "##.####/", 0 ) ) return 0;
	if ( string_pattern( s, "doi:##.####/", 0 ) ) return 4;
	if ( string_pattern( s, "doi: ##.####/", 0 ) ) return 5;
	if ( string_pattern( s, "doi: DOI: ##.####/", 0 ) ) return 10;
	if ( string_pattern( s, "https://doi.org/##.####/", 0 ) ) return 16;
	return -1;
}

/* determine if string has the header of a Universal Resource Identifier
 *
 * returns -1, if not true
 * returns offset that skips over the URI scheme, if true
 */
int
is_uri_remote_scheme( char *p )
{
	char *scheme[]   = { "http:", "https:", "ftp:", "git:", "gopher:" };
	int  schemelen[] = { 5,       6,        4,      4,      7         };
        int i, nschemes = sizeof( scheme ) / sizeof( scheme[0] );
        for ( i=0; i<nschemes; ++i ) {
                if ( !strncasecmp( p, scheme[i], schemelen[i] ) ) return schemelen[i];
        }
        return -1;
}

int
is_reference_database( char *p )
{
	char *scheme[]   = { "arXiv:", "pubmed:", "medline:", "isi:" };
	int  schemelen[] = { 6,        7,         8,          4      };
        int i, nschemes = sizeof( scheme ) / sizeof( scheme[0] );
        for ( i=0; i<nschemes; ++i ) {
                if ( !strncasecmp( p, scheme[i], schemelen[i] ) ) return schemelen[i];
        }
        return -1;
}

/* many fields have been abused to embed URLs, DOIs, etc. */
int
is_embedded_link( char *s )
{
	if ( is_uri_remote_scheme( s )  != -1 ) return 1;
	if ( is_reference_database( s ) != -1 ) return 1;
	if ( is_doi( s ) !=-1 ) return 1;
	return 0;
}

typedef struct url_t {
	char *tag;
	char *prefix;
	int offset;
} url_t;

static url_t prefixes[] = {
	/*              00000000001111111112222222222333333333344444444445 */
	/*              12345678901234567890123456789012345678901234567890 */
	{ "ARXIV",     "http://arxiv.org/abs/",                     21 },
	{ "DOI",       "https://doi.org/",                          16 },
	{ "JSTOR",     "http://www.jstor.org/stable/",              28 },
	{ "MRNUMBER",  "http://www.ams.org/mathscinet-getitem?mr=", 41 },
	{ "PMID",      "http://www.ncbi.nlm.nih.gov/pubmed/",       35 },
	{ "PMC",       "http://www.ncbi.nlm.nih.gov/pmc/articles/", 41 },
	{ "ISIREFNUM", "isi:",                                       4 },
};
static int nprefixes = sizeof( prefixes ) / sizeof( prefixes[0] );

/* do not add, but recognize */
static url_t extraprefixes[] = {
	/*              00000000001111111112222222222333333333344444444445 */
	/*              12345678901234567890123456789012345678901234567890 */
	{ "ARXIV",     "arXiv:",                                     6 },
	{ "DOI",       "http://dx.doi.org/",                        18 },
	{ "JSTOR",     "jstor:",                                     6 },
	{ "PMID",      "pmid:",                                      5 },
	{ "PMID",      "pubmed:",                                    7 },
	{ "PMC",       "pmc:",                                       4 },
	{ "URL",       "\\urllink",                                  8 },
	{ "URL",       "\\url",                                      4 },
};
static int nextraprefixes = sizeof( extraprefixes ) / sizeof( extraprefixes[0] );

static int
find_prefix( char *s, url_t *p, int np )
{
	int i;

	for ( i=0; i<np; ++i )
		if ( !strncmp( p[i].prefix, s, p[i].offset ) ) return i;

	return -1;
}

int
urls_split_and_add( char *value_in, fields *out, int lvl_out )
{
	int n, fstatus, status = BIBL_OK;
	char *tag = "URL";
	int offset = 0;

	n = find_prefix( value_in, prefixes, nprefixes );
	if ( n!=-1 ) {
		tag    = prefixes[n].tag;
		offset = prefixes[n].offset;
	} else {
		n = find_prefix( value_in, extraprefixes, nextraprefixes );
		if ( n!=-1 ) {
			tag    = extraprefixes[n].tag;
			offset = extraprefixes[n].offset;
		}
	}

	fstatus = fields_add( out, tag, &(value_in[offset]), lvl_out );
	if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;

	return status;
}

/* urls_add_type()
 *
 * Append urls of a specific type with a specific prefix (which can be empty).
 * We don't allow duplications here.
 *
 */
static int
urls_merge_and_add_type( fields *out, char *tag_out, int lvl_out, char *prefix, vplist *values )
{
	int fstatus, status = BIBL_OK;
	vplist_index i;
	str url;

	str_init( &url );

	for ( i=0; i<values->n; ++i ) {
		str_strcpyc( &url, prefix );
		str_strcatc( &url, ( char * ) vplist_get( values, i ) );
		fstatus = fields_add( out, tag_out, str_cstr( &url ), lvl_out );
		if ( fstatus!=FIELDS_OK ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}

	}
out:
	str_free( &url );
	return status;
}

/*
 * urls_merge_and_add()
 *
 * Append urls of types controlled by the list type and automatically append appropriate
 * prefixes. If no prefix is found for the entry, don't add one (e.g. "URL" entries).
 *
 * Control of the types to be added by list type is necessary as some reference formats
 * like bibtex ought to do special things with DOI, ARXIV, MRNUMBER, and the like.
 */
int
urls_merge_and_add( fields *in, int lvl_in, fields *out, char *tag_out, int lvl_out, slist *types )
{
	int i, j, status = BIBL_OK;
	char *tag, *prefix, *empty="";
	vplist a;

	vplist_init( &a );

	for ( i=0; i<types->n; ++i ) {

		tag = slist_cstr( types, i );

		/* ...look for data of requested type; if not found skip */
		vplist_empty( &a );
		fields_findv_each( in, lvl_in, FIELDS_CHRP, &a, tag );
		if ( a.n==0 ) continue;

		/* ...find the prefix (if present) */
		prefix = empty;
		for ( j=0; j<nprefixes; ++j ) {
			if ( !strcmp( prefixes[j].tag, tag ) ) {
				prefix = prefixes[j].prefix;
				break; /* take the first prefix in the list */
			}
		}

		/* ...append all data of this type */
		status = urls_merge_and_add_type( out, tag_out, lvl_out, prefix, &a );
		if ( status!=BIBL_OK ) goto out;
	}

out:
	vplist_free( &a );

	return status;
}
