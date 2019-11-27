/*
 * modsin.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "is_ws.h"
#include "str.h"
#include "str_conv.h"
#include "xml.h"
#include "xml_encoding.h"
#include "fields.h"
#include "name.h"
#include "reftypes.h"
#include "modstypes.h"
#include "bu_auth.h"
#include "marc_auth.h"
#include "url.h"
#include "iso639_1.h"
#include "iso639_2.h"
#include "iso639_3.h"
#include "bibutils.h"
#include "bibformats.h"

static int modsin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset );
static int modsin_processf( fields *medin, const char *data, const char *filename, long nref, param *p );

/*****************************************************
 PUBLIC: void modsin_initparams()
*****************************************************/
int
modsin_initparams( param *pm, const char *progname )
{

	pm->readformat       = BIBL_MODSIN;
	pm->format_opts      = 0;
	pm->charsetin        = BIBL_CHARSET_UNICODE;
	pm->charsetin_src    = BIBL_SRC_DEFAULT;
	pm->latexin          = 0;
	pm->utf8in           = 1;
	pm->xmlin            = 1;
	pm->nosplittitle     = 0;
	pm->verbose          = 0;
	pm->addcount         = 0;
	pm->singlerefperfile = 0;
	pm->output_raw       = BIBL_RAW_WITHMAKEREFID |
	                      BIBL_RAW_WITHCHARCONVERT;

	pm->readf    = modsin_readf;
	pm->processf = modsin_processf;
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
 PUBLIC: int modsin_processf()
*****************************************************/

static char modsns[]="mods";

static int
modsin_detailr( xml *node, str *value )
{
	int status = BIBL_OK;
	if ( xml_has_value( node ) ) {
		if ( value->len ) str_addchar( value, ' ' );
		str_strcat( value, xml_value( node ) );
		if ( str_memerr( value ) ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = modsin_detailr( node->down, value );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next )
		status = modsin_detailr( node->next, value );
	return status;
}

static int
modsin_detail( xml *node, fields *info, int level )
{
	str type, value, *tp;
	int fstatus, status = BIBL_OK;
	if ( node->down ) {
		strs_init( &type, &value, NULL );
		tp = xml_attribute( node, "type" );
		if ( tp ) {
			str_strcpy( &type, tp );
			str_toupper( &type );
			if ( str_memerr( &type ) ) goto out;
		}
		status = modsin_detailr( node->down, &value );
		if ( status!=BIBL_OK ) goto out;
		if ( type.data && !strcasecmp( type.data, "PAGE" ) ) {
			fstatus = fields_add( info, "PAGES:START", value.data, level );
		} else {
			fstatus = fields_add( info, type.data, value.data, level );
		}
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
out:
		strs_free( &type, &value, NULL );
	}
	return status;
}

static int
modsin_date( xml *node, fields *info, int level, int part )
{
	int fstatus, status = BIBL_OK;
	const char *tag, *p;
	str s;

	str_init( &s );

	p = xml_value_cstr( node );

	if ( p ) {

		p = str_cpytodelim( &s, skip_ws( p ), "-", 1 );
		if ( str_memerr( &s ) ) { status = BIBL_ERR_MEMERR; goto out; }
		if ( str_has_value( &s ) ) {
			tag = ( part ) ? "PARTDATE:YEAR" : "DATE:YEAR";
			fstatus =  fields_add( info, tag, str_cstr( &s ), level );
			if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
		}

		p = str_cpytodelim( &s, skip_ws( p ), "-", 1 );
		if ( str_memerr( &s ) ) { status = BIBL_ERR_MEMERR; goto out; }
		if ( str_has_value( &s ) ) {
			tag = ( part ) ? "PARTDATE:MONTH" : "DATE:MONTH";
			fstatus =  fields_add( info, tag, str_cstr( &s ), level );
			if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
		}

		(void) str_cpytodelim( &s, skip_ws( p ), "", 0 );
		if ( str_memerr( &s ) ) { status = BIBL_ERR_MEMERR; goto out; }
		if ( str_has_value( &s ) ) {
			tag = ( part ) ? "PARTDATE:DAY" : "DATE:DAY";
			fstatus =  fields_add( info, tag, str_cstr( &s ), level );
			if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
		}

	}

out:
	str_free( &s );
	return status;
}

static int
modsin_pager( xml *node, str *sp, str *ep, str *tp, str *lp )
{
	int status = BIBL_OK;
	if ( xml_tag_matches_has_value( node, "start" ) ) {
		str_strcpy( sp, xml_value( node ) );
		if ( str_memerr( sp ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "end" ) ) {
		str_strcpy( ep, xml_value( node ) );
		if ( str_memerr( ep ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "total" ) ) {
		str_strcpy( tp, xml_value( node ) );
		if ( str_memerr( tp ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "list" ) ) {
		str_strcpy( lp, xml_value( node ) );
		if ( str_memerr( lp ) ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = modsin_pager( node->down, sp, ep, tp, lp );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next )
		status = modsin_pager( node->next, sp, ep, tp, lp );
	return status;
}

static int
modsin_page( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	str sp, ep, tp, lp;
	xml *dnode = node->down;

	if ( !dnode ) return BIBL_OK;

	strs_init( &sp, &ep, &tp, &lp, NULL );

	status = modsin_pager( dnode, &sp, &ep, &tp, &lp );
	if ( status!=BIBL_OK ) goto out;

	if ( str_has_value( &sp ) || str_has_value( &ep ) ) {
		if ( str_has_value( &sp ) ) {
			fstatus = fields_add( info, "PAGES:START", str_cstr( &sp ), level );
			if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
		}
		if ( str_has_value( &ep ) ) {
			fstatus = fields_add( info, "PAGES:STOP", str_cstr( &ep ), level );
			if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
		}
	} else if ( str_has_value( &lp ) ) {
		fstatus = fields_add( info, "PAGES:START", str_cstr( &lp ), level );
		if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
	}
	if ( str_has_value( &tp ) ) {
		fstatus = fields_add( info, "PAGES:TOTAL", str_cstr( &tp ), level );
		if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
	}
out:
	strs_free( &sp, &ep, &tp, &lp, NULL );
	return status;
}

static int
modsin_titler( xml *node, str *title, str *subtitle )
{
	int status = BIBL_OK;
	if ( xml_tag_matches_has_value( node, "title" ) ) {
		if ( str_has_value( title ) ) str_strcatc( title, " : " );
		str_strcat( title, xml_value( node ) );
		if ( str_memerr( title ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "subTitle" ) ) {
		str_strcat( subtitle, xml_value( node ) );
		if ( str_memerr( subtitle ) ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = modsin_titler( node->down, title, subtitle );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next )
		status = modsin_titler( node->next, title, subtitle );
	return status;
}

static int
modsin_title( xml *node, fields *info, int level )
{
	char *titletag[2][2] = {
		{ "TITLE",    "SHORTTITLE" },
		{ "SUBTITLE", "SHORTSUBTITLE" },
	};
	int fstatus, status = BIBL_OK;
	str title, subtitle;
	xml *dnode;
	int abbr;

	dnode = node->down;
	if ( !dnode ) return status;

	strs_init( &title, &subtitle, NULL );
	abbr = xml_tag_has_attribute( node, "titleInfo", "type", "abbreviated" );

	status = modsin_titler( dnode, &title, &subtitle );
	if ( status!=BIBL_OK ) goto out;

	if ( str_has_value( &title ) ) {
		fstatus = fields_add( info, titletag[0][abbr], str_cstr( &title ), level );
		if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
	}

	if ( str_has_value( &subtitle ) ) {
		fstatus = fields_add( info, titletag[1][abbr], str_cstr( &subtitle ), level );
		if ( fstatus!=FIELDS_OK ) { status = BIBL_ERR_MEMERR; goto out; }
	}

out:
	strs_free( &title, &subtitle, NULL );
	return status;
}

/* modsin_marcrole_convert()
 *
 * Map MARC-authority roles for people or organizations associated
 * with a reference to internal roles.
 *
 * Take input strings with roles separated by '|' characters, e.g.
 * "author" or "author|creator" or "edt" or "editor|edt".
 */
static int
modsin_marcrole_convert( str *s, char *suffix, str *out )
{
	int i, sstatus, status = BIBL_OK;
	slist tokens;
	char *p;

	slist_init( &tokens );

	/* ...default to author on an empty string */
	if ( str_is_empty( s ) ) {
		str_strcpyc( out, "AUTHOR" );
	}

	else {
		sstatus = slist_tokenize( &tokens, s, "|", 1 );
		if ( sstatus!=SLIST_OK ) {
			status = BIBL_ERR_MEMERR;
			goto done;
		}
		/* ...take first match */
		for ( i=0; i<tokens.n; ++i ) {
			p = marc_convertrole( slist_cstr( &tokens, i ) );
			if ( p ) {
				str_strcpyc( out, p );
				goto done;
			}
		}
		/* ...otherwise just copy input */
		str_strcpy( out, slist_str( &tokens, 0 ) );
		str_toupper( out );
	}

done:
	if ( suffix ) str_strcatc( out, suffix );
	slist_free( &tokens );
	if ( str_memerr( out ) ) return BIBL_ERR_MEMERR;
	return status;
}

static int
modsin_asis_corp_r( xml *node, str *name, str *role )
{
	int status = BIBL_OK;
	if ( xml_tag_matches_has_value( node, "namePart" ) ) {
		str_strcpy( name, xml_value( node ) );
		if ( str_memerr( name ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "roleTerm" ) ) {
		if ( role->len ) str_addchar( role, '|' );
		str_strcat( role, xml_value( node ) );
		if ( str_memerr( role ) ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = modsin_asis_corp_r( node->down, name, role );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next )
		status = modsin_asis_corp_r( node->next, name, role );
	return status;
}

static int
modsin_asis_corp( xml *node, fields *info, int level, char *suffix )
{
	int fstatus, status = BIBL_OK;
	str name, roles, role_out;
	xml *dnode = node->down;
	if ( dnode ) {
		strs_init( &name, &roles, &role_out, NULL );
		status = modsin_asis_corp_r( dnode, &name, &roles );
		if ( status!=BIBL_OK ) goto out;
		status = modsin_marcrole_convert( &roles, suffix, &role_out );
		if ( status!=BIBL_OK ) goto out;
		fstatus = fields_add( info, str_cstr( &role_out ), str_cstr( &name ), level );
		if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
out:
		strs_free( &name, &roles, &role_out, NULL );
	}
	return status;
}

static int
modsin_roler( xml *node, str *roles )
{
	if ( xml_has_value( node ) ) {
		if ( roles->len ) str_addchar( roles, '|' );
		str_strcat( roles, xml_value( node ) );
	}
	if ( str_memerr( roles ) ) return BIBL_ERR_MEMERR;
	else return BIBL_OK;
}

static int
modsin_personr( xml *node, str *familyname, str *givenname, str *suffix )
{
	int status = BIBL_OK;

	if ( !xml_has_value( node ) ) return status;

	if ( xml_tag_has_attribute( node, "namePart", "type", "family" ) ) {
		if ( str_has_value( familyname ) ) str_addchar( familyname, ' ' );
		str_strcat( familyname, xml_value( node ) );
		if ( str_memerr( familyname ) ) status = BIBL_ERR_MEMERR;
	}

	else if ( xml_tag_has_attribute( node, "namePart", "type", "suffix"         ) ||
	          xml_tag_has_attribute( node, "namePart", "type", "termsOfAddress" ) ) {
		if ( str_has_value( suffix ) ) str_addchar( suffix, ' ' );
		str_strcat( suffix, xml_value( node ) );
		if ( str_memerr( suffix ) ) status = BIBL_ERR_MEMERR;
	}

	else if ( xml_tag_has_attribute( node, "namePart", "type", "date" ) ) {
		/* no nothing */
	}

	else {
		if ( str_has_value( givenname ) ) str_addchar( givenname, '|' );
		str_strcat( givenname, xml_value( node ) );
		if ( str_memerr( givenname ) ) status = BIBL_ERR_MEMERR;
	}

	return status;
}

static int
modsin_person( xml *node, fields *info, int level )
{
	str familyname, givenname, name, suffix, roles, role_out;
	int fstatus, status = BIBL_OK;
	xml *dnode, *rnode;

	dnode = node->down;
	if ( !dnode ) return status;

	strs_init( &name, &familyname, &givenname, &suffix, &roles, &role_out, NULL );

	while ( dnode ) {

		if ( xml_tag_matches( dnode, "namePart" ) ) {
			status = modsin_personr( dnode, &familyname, &givenname, &suffix );
			if ( status!=BIBL_OK ) goto out;
		}

		else if ( xml_tag_matches( dnode, "role" ) ) {
			rnode = dnode->down;
			while ( rnode ) {
				if ( xml_tag_matches( rnode, "roleTerm" ) ) {
					status = modsin_roler( rnode, &roles );
					if ( status!=BIBL_OK ) goto out;
				}
				rnode = rnode->next;
			}
		}

		dnode = dnode->next;

	}

	/*
	 * Handle:
	 *          <namePart type='given'>Noah A.</namePart>
	 *          <namePart type='family'>Smith</namePart>
	 * without mangling the order of "Noah A."
	 */
	if ( str_has_value( &familyname ) ) {
		str_strcpy( &name, &familyname );
		if ( givenname.len ) {
			str_addchar( &name, '|' );
			str_strcat( &name, &givenname );
		}
	}

	/*
	 * Handle:
	 *          <namePart>Noah A. Smith</namePart>
	 * with name order mangling.
	 */
	else {
		if ( str_has_value( &givenname ) )
			name_parse( &name, &givenname, NULL, NULL );
	}

	if ( str_has_value( &suffix ) ) {
		str_strcatc( &name, "||" );
		str_strcat( &name, &suffix );
	}

	if ( str_memerr( &name ) ) {
		status=BIBL_ERR_MEMERR;
		goto out;
	}

	status = modsin_marcrole_convert( &roles, NULL, &role_out );
	if ( status!=BIBL_OK ) goto out;

	fstatus = fields_add_can_dup( info, str_cstr( &role_out ), str_cstr( &name ), level );
	if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;

out:
	strs_free( &name, &familyname, &givenname, &suffix, &roles, &role_out, NULL );
	return status;
}

static int
modsin_placeterm_text( xml *node, fields *info, int level, int school )
{
	char address_tag[] = "ADDRESS";
	char school_tag[]  = "SCHOOL";
	char *tag;
	int fstatus;

	tag = ( school ) ? school_tag : address_tag;

	fstatus = fields_add( info, tag, xml_value_cstr( node ), level );
	if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;

	return BIBL_OK;
}

static int
modsin_placeterm_code( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	str s, *auth;

	str_init( &s );

	auth = xml_attribute( node, "authority" );
	if ( auth && auth->len ) {
		str_strcpy( &s, auth );
		str_addchar( &s, '|' );
	}
	str_strcat( &s, xml_value( node ) );

	if ( str_memerr( &s ) ) {
		status = BIBL_ERR_MEMERR;
		goto out;
	}

	fstatus = fields_add( info, "CODEDADDRESS", str_cstr( &s ), level );
	if ( fstatus!=FIELDS_OK ) status = BIBL_ERR_MEMERR;
out:
	str_free( &s );
	return status;
}

static int
modsin_placeterm( xml *node, fields *info, int level, int school )
{
	int status = BIBL_OK;
	str *type;

	type = xml_attribute( node, "type" );
	if ( str_has_value( type ) ) {
		if ( !strcmp( str_cstr( type ), "text" ) )
			status = modsin_placeterm_text( node, info, level, school );
		else if ( !strcmp( str_cstr( type ), "code" ) )
			status = modsin_placeterm_code( node, info, level );
	}

	return status;
}

static int
modsin_placer( xml *node, fields *info, int level, int school )
{
	int status = BIBL_OK;

	if ( xml_tag_has_attribute( node, "place", "type", "school" ) ) {
		school = 1;
	} else if ( xml_tag_matches( node, "placeTerm" ) ) {
		status = modsin_placeterm( node, info, level, school );
	}

	if ( node->down ) {
		status = modsin_placer( node->down, info, level, school );
		if ( status!=BIBL_OK ) return status;
	}

	if ( node->next ) status = modsin_placer( node->next, info, level, school );

	return status;
}

static int
modsin_origininfor( xml *node, fields *info, int level, str *pub, str *add, str *addc, str *ed, str *iss )
{
	int status = BIBL_OK;

	if ( xml_tag_matches( node, "dateIssued" ) ) {
		status = modsin_date( node, info, level, 0 );
	} else if ( xml_tag_matches( node, "place" ) ) {
		status = modsin_placer( node, info, level, 0 );
	} else if ( xml_tag_matches_has_value( node, "publisher" ) ) {
		str_strcat( pub, xml_value( node ) );
		if ( str_memerr( pub ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "edition" ) ) {
		str_strcat( ed, xml_value( node ) );
		if( str_memerr( ed ) ) return BIBL_ERR_MEMERR;
	} else if ( xml_tag_matches_has_value( node, "issuance" ) ) {
		str_strcat( iss, xml_value( node ) );
		if ( str_memerr( iss ) ) return BIBL_ERR_MEMERR;
	}
	if ( status!=BIBL_OK ) return status;

	if ( node->down ) {
		status = modsin_origininfor( node->down, info, level, pub, add, addc, ed, iss );
		if ( status!=BIBL_OK ) return status;
	}

	if ( node->next )
		status = modsin_origininfor( node->next, info, level, pub, add, addc, ed, iss );

	return status;
}

static int
modsin_origininfo( xml *node, fields *info, int level )
{
	str publisher, address, addcode, edition, issuance;
	int fstatus, status = BIBL_OK;
	if ( node->down ) {
		strs_init( &publisher, &address, &addcode, &edition, &issuance, NULL );
		status = modsin_origininfor( node->down, info, level, &publisher, 
				&address, &addcode, &edition, &issuance );
		if ( status!=BIBL_OK ) goto out;
		if ( str_has_value( &publisher ) ) {
			fstatus = fields_add( info, "PUBLISHER", str_cstr( &publisher ), level );
			if ( fstatus!=FIELDS_OK ) { status=BIBL_ERR_MEMERR; goto out; }
		}
		if ( str_has_value( &address ) ) {
			fstatus = fields_add( info, "ADDRESS", str_cstr( &address ), level );
			if ( fstatus!=FIELDS_OK ) { status=BIBL_ERR_MEMERR; goto out; }
		}
		if ( str_has_value( &addcode ) ) {
			fstatus = fields_add( info, "CODEDADDRESS", str_cstr( &addcode ), level );
			if ( fstatus!=FIELDS_OK ) { status=BIBL_ERR_MEMERR; goto out; }
		}
		if ( str_has_value( &edition ) ) {
			fstatus = fields_add( info, "EDITION", str_cstr( &edition ), level );
			if ( fstatus!=FIELDS_OK ) { status=BIBL_ERR_MEMERR; goto out; }
		}
		if ( str_has_value( &issuance ) ) {
			fstatus = fields_add( info, "ISSUANCE", str_cstr( &issuance ), level );
			if ( fstatus!=FIELDS_OK ) { status=BIBL_ERR_MEMERR; goto out; }
		}
out:
		strs_free( &publisher, &address, &addcode, &edition, &issuance, NULL );
	}
	return status;
}

static int
modsin_subjectr( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	if ( xml_tag_has_attribute( node, "topic", "class", "primary" ) && xml_has_value( node ) ) {
		fstatus = fields_add( info, "EPRINTCLASS", xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	else if ( xml_tag_matches_has_value( node, "topic" ) ) {
		fstatus = fields_add( info, "KEYWORD", xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	else if ( xml_tag_matches_has_value( node, "geographic" ) ) {
		fstatus = fields_add( info, "KEYWORD", xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = modsin_subjectr( node->down, info, level );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = modsin_subjectr( node->next, info, level );
	return status;
}

static int
modsin_subject( xml *node, fields *info, int level )
{
	int status = BIBL_OK;
	if ( node->down ) status = modsin_subjectr( node->down, info, level );
	return status;
}

static int
modsin_id1( xml *node, fields *info, int level )
{
	int fstatus;
	str *ns;
	ns = xml_attribute( node, "ID" );
	if ( str_has_value( ns ) ) {
		fstatus = fields_add( info, "REFNUM", str_cstr( ns ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	return BIBL_OK;
}

/* modsin_genre()
 *
 * MARC authority terms tagged with "GENRE:MARC"
 * bibutils authority terms tagged with "GENRE:BIBUTILS"
 * unknown terms tagged with "GENRE:UNKNOWN"
 */
static int
modsin_genre( xml *node, fields *info, int level )
{
	int fstatus;
	char *d;

	if ( !xml_has_value( node ) ) return BIBL_OK;

	d = xml_value_cstr( node );

	/* ...handle special genres in KTH DivA */
	if ( !strcmp( d, "conferenceProceedings" ) || !strcmp( d, "conferencePaper" ) )
		d = "conference publication";
	else if ( !strcmp( d, "artisticOutput" ) || !strcmp( d, "other" ) )
		d = "miscellaneous";
	else if ( !strcmp( d, "studentThesis" ) )
		d = "thesis";
	else if ( !strcmp( d, "monographDoctoralThesis" ) )
		d = "Ph.D. thesis";
	else if ( !strcmp( d, "comprehensiveDoctoralThesis" ) )
		d = "Ph.D. thesis";
	else if ( !strcmp( d, "monographLicentiateThesis" ) )
		d = "Licentiate thesis";
	else if ( !strcmp( d, "comprehensiveLicentiateThesis" ) )
		d = "Licentiate thesis";

	if ( is_marc_genre( d ) )
		fstatus = fields_add( info, "GENRE:MARC", d, level );
	else if ( is_bu_genre( d ) )
		fstatus = fields_add( info, "GENRE:BIBUTILS", d, level );
	else
		fstatus = fields_add( info, "GENRE:UNKNOWN", d, level );

	if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	else return BIBL_OK;
}

/* in MODS version 3.5
 * <languageTerm type="text">....</languageTerm>
 * <languageTerm type="code" authority="xxx">...</languageTerm>
 * xxx = rfc3066
 * xxx = iso639-2b
 * xxx = iso639-3
 * xxx = rfc4646
 * xxx = rfc5646
 */
static int
modsin_languager( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	char *d = NULL;
	if ( xml_tag_matches( node, "languageTerm" ) ) {
		if ( xml_has_value( node ) ) {
			if ( xml_has_attribute( node, "type", "code" ) ) {
				if ( xml_has_attribute( node, "authority", "iso639-1" ) )
					d = iso639_1_from_code( xml_value_cstr( node ) );
				else if ( xml_has_attribute( node, "authority", "iso639-2b" ) )
					d = iso639_2_from_code( xml_value_cstr( node ) );
				else if ( xml_has_attribute( node, "authority", "iso639-3" ))
					d = iso639_3_from_code( xml_value_cstr( node ) );
			}
			if ( !d ) d  = xml_value_cstr( node );
			fstatus = fields_add( info, "LANGUAGE", d, level );
			if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
	}
	if ( node->next ) status = modsin_languager( node->next, info, level );
	return status;
}

static int
modsin_language( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	/* Old versions of MODS had <language>English</language> */
	if ( xml_has_value( node ) ) {
		fstatus = fields_add( info, "LANGUAGE", xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	/* New versions of MODS have <language><languageTerm>English</languageTerm></language> */
	if ( node->down ) status = modsin_languager( node->down, info, level );
	return status;
}

static int
modsin_simple( xml *node, fields *info, char *tag, int level )
{
	int fstatus;
	if ( xml_has_value( node ) ) {
		fstatus = fields_add( info, tag, xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	return BIBL_OK;
}

static int
modsin_locationr( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	char *url        = "URL";
	char *fileattach = "FILEATTACH";
	char *tag=NULL;

	if ( xml_tag_matches( node, "url" ) ) {
		if ( xml_has_attribute( node, "access", "raw object" ) )
			tag = fileattach;
		else
			tag = url;
	}
	else if ( xml_tag_matches( node, "physicalLocation" ) ) {
		if ( xml_has_attribute( node, "type", "school" ) )
			tag = "SCHOOL";
		else
			tag = "LOCATION";
	}

	if ( tag == url ) {
		status = urls_split_and_add( xml_value_cstr( node ), info, level );
		if ( status!=BIBL_OK ) return status;
	}
	else if ( tag ) {
		fstatus = fields_add( info, tag, xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}

	if ( node->down ) {
		status = modsin_locationr( node->down, info, level );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = modsin_locationr( node->next, info, level );
	return status;
}

static int
modsin_location( xml *node, fields *info, int level )
{
	int status = BIBL_OK;
	if ( node->down ) status = modsin_locationr( node->down, info, level );
	return status;
}

static int
modsin_descriptionr( xml *node, str *s )
{
	int status = BIBL_OK;
	if ( xml_tag_matches( node, "extent" ) ||
	     xml_tag_matches( node, "note" ) ) {
		str_strcpy( s, &(node->value) );
		if ( str_memerr( s ) ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) {
		status = modsin_descriptionr( node->down, s );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = modsin_descriptionr( node->next, s );
	return status;
}

static int
modsin_description( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	str s;
	str_init( &s );
	if ( node->down ) {
		status = modsin_descriptionr( node->down, &s );
		if ( status!=BIBL_OK ) goto out;
	} else {
		if ( node->value.len > 0 )
			str_strcpy( &s, &(node->value) );
		if ( str_memerr( &s ) ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}
	}
	if ( str_has_value( &s ) ) {
		fstatus = fields_add( info, "DESCRIPTION", str_cstr( &s ), level );
		if ( fstatus!=FIELDS_OK ) {
			status = BIBL_ERR_MEMERR;
			goto out;
		}
	}
out:
	str_free( &s );
	return status;
}

static int
modsin_partr( xml *node, fields *info, int level )
{
	int status = BIBL_OK;
	if ( xml_tag_matches( node, "detail" ) )
		status = modsin_detail( node, info, level );
	else if ( xml_tag_has_attribute( node, "extent", "unit", "page" ) )
		status = modsin_page( node, info, level );
	else if ( xml_tag_has_attribute( node, "extent", "unit", "pages" ) )
		status = modsin_page( node, info, level );
	else if ( xml_tag_matches( node, "date" ) )
		status = modsin_date( node, info, level, 1 );
	if ( status!=BIBL_OK ) return status;
	if ( node->next ) status = modsin_partr( node->next, info, level );
	return status;
}

static int
modsin_part( xml *node, fields *info, int level )
{
	if ( node->down ) return modsin_partr( node->down, info, level );
	return BIBL_OK;
}

/* <classification authority="lcc">Q3 .A65</classification> */
static int
modsin_classification( xml *node, fields *info, int level )
{
	int fstatus, status = BIBL_OK;
	char *tag;
	if ( xml_has_value( node ) ) {
		if ( xml_tag_has_attribute( node, "classification", "authority", "lcc" ) )
			tag = "LCC";
		else
			tag = "CLASSIFICATION";
		fstatus = fields_add( info, tag, xml_value_cstr( node ), level );
		if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
	}
	if ( node->down ) status = modsin_classification( node->down, info, level );
	return status;
}

static int
modsin_recordinfo( xml *node, fields *info, int level )
{
	int fstatus;
	xml *curr;

	/* extract recordIdentifier */
	curr = node;
	while ( curr ) {
		if ( xml_tag_matches_has_value( curr, "recordIdentifier" ) ) {
			fstatus = fields_add( info, "REFNUM", xml_value_cstr( curr ), level );
			if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
		curr = curr->next;
	}
	return BIBL_OK;
}

static int
modsin_identifier( xml *node, fields *info, int level )
{
	convert ids[] = {
		{ "citekey",       "REFNUM",      0, 0 },
		{ "issn",          "ISSN",        0, 0 },
		{ "coden",         "CODEN",       0, 0 },
		{ "isbn",          "ISBN",        0, 0 },
		{ "doi",           "DOI",         0, 0 },
		{ "url",           "URL",         0, 0 },
		{ "uri",           "URL",         0, 0 },
		{ "pmid",          "PMID",        0, 0 },
		{ "pubmed",        "PMID",        0, 0 },
		{ "medline",       "MEDLINE",     0, 0 },
		{ "pmc",           "PMC",         0, 0 },
		{ "arXiv",         "ARXIV",       0, 0 },
		{ "MRnumber",      "MRNUMBER",    0, 0 },
		{ "pii",           "PII",         0, 0 },
		{ "isi",           "ISIREFNUM",   0, 0 },
		{ "serial number", "SERIALNUMBER",0, 0 },
		{ "accessnum",     "ACCESSNUM",   0, 0 },
		{ "jstor",         "JSTOR",       0, 0 },
	};
	int i, fstatus, n = sizeof( ids ) / sizeof( ids[0] );
	if ( node->value.len==0 ) return BIBL_OK;
	for ( i=0; i<n; ++i ) {
		if ( xml_tag_has_attribute( node, "identifier", "type", ids[i].mods ) ) {
			fstatus = fields_add( info, ids[i].internal, xml_value_cstr( node ), level );
			if ( fstatus!=FIELDS_OK ) return BIBL_ERR_MEMERR;
		}
	}
	return BIBL_OK;
}

static int
modsin_mods( xml *node, fields *info, int level )
{
	convert simple[] = {
		{ "note",            "NOTES",    0, 0 },
		{ "abstract",        "ABSTRACT", 0, 0 },
		{ "bibtex-annote",   "ANNOTE",   0, 0 },
		{ "typeOfResource",  "RESOURCE", 0, 0 },
		{ "tableOfContents", "CONTENTS", 0, 0 },
	};
	int nsimple = sizeof( simple ) / sizeof( simple[0] );
	int i, found = 0, status = BIBL_OK;

	for ( i=0; i<nsimple && found==0; i++ ) {
		if ( xml_tag_matches( node, simple[i].mods ) ) {
			status = modsin_simple( node, info, simple[i].internal, level );
			if ( status!=BIBL_OK ) return status;
			found = 1;
		}
	}

	if ( !found ) {
		if ( xml_tag_matches( node, "titleInfo" ) )
			modsin_title( node, info, level );
		else if ( xml_tag_has_attribute( node, "name", "type", "personal" ) )
			status = modsin_person( node, info, level );
		else if ( xml_tag_has_attribute( node, "name", "type", "corporate" ) )
			status = modsin_asis_corp( node, info, level, ":CORP" );
		else if ( xml_tag_matches( node, "name" ) )
			status = modsin_asis_corp( node, info, level, ":ASIS" );
		else if ( xml_tag_matches( node, "recordInfo" ) && node->down )
			status = modsin_recordinfo( node->down, info, level );
		else if  ( xml_tag_matches( node, "part" ) )
			modsin_part( node, info, level );
		else if ( xml_tag_matches( node, "identifier" ) )
			status = modsin_identifier( node, info, level );
		else if ( xml_tag_matches( node, "originInfo" ) )
			status = modsin_origininfo( node, info, level );
		else if ( xml_tag_matches( node, "language" ) )
			status = modsin_language( node, info, level );
		else if ( xml_tag_matches( node, "genre" ) )
			status = modsin_genre( node, info, level );
		else if ( xml_tag_matches( node, "date" ) )
			status = modsin_date( node, info, level, 0 );
		else if ( xml_tag_matches( node, "subject" ) )
			status = modsin_subject( node, info, level );
		else if ( xml_tag_matches( node, "classification" ) )
			status = modsin_classification( node, info, level );
		else if ( xml_tag_matches( node, "location" ) )
			status = modsin_location( node, info, level );
		else if ( xml_tag_matches( node, "physicalDescription" ) )
			status = modsin_description( node, info, level );
		else if ( xml_tag_has_attribute( node, "relatedItem", "type", "host" ) ||
			  xml_tag_has_attribute( node, "relatedItem", "type", "series" ) ) {
			if ( node->down ) status = modsin_mods( node->down, info, level+1 );
		}
		else if ( xml_tag_has_attribute( node, "relatedItem", "type", "original" ) ) {
			if ( node->down ) status = modsin_mods( node->down, info, LEVEL_ORIG );
		}

		if ( status!=BIBL_OK ) return status;
	}

	if ( node->next ) status = modsin_mods( node->next, info, level );

	return status;
}

static int
modsin_assembleref( xml *node, fields *info )
{
	int status = BIBL_OK;
	if ( xml_tag_matches( node, "mods" ) ) {
		status = modsin_id1( node, info, 0 );
		if ( status!=BIBL_OK ) return status;
		if ( node->down ) {
			status = modsin_mods( node->down, info, 0 );
			if ( status!=BIBL_OK ) return status;
		}
	} else if ( node->down ) {
		status = modsin_assembleref( node->down, info );
		if ( status!=BIBL_OK ) return status;
	}
	if ( node->next ) status = modsin_assembleref( node->next, info );
	return status;
}

static int
modsin_processf( fields *modsin, const char *data, const char *filename, long nref, param *p )
{
	int status;
	xml top;

	xml_init( &top );
	xml_parse( data, &top );
	status = modsin_assembleref( &top, modsin );
	xml_free( &top );

	if ( status==BIBL_OK ) return 1;
	else return 0;
}

/*****************************************************
 PUBLIC: int modsin_readf()
*****************************************************/

static char *
modsin_startptr( char *p )
{
	char *startptr;
	startptr = xml_find_start( p, "mods:mods" );
	if ( startptr ) {
		/* set namespace if found */
		xml_pns = modsns;
	} else {
		startptr = xml_find_start( p, "mods" );
		if ( startptr ) xml_pns = NULL;
	}
	return startptr;
}

static char *
modsin_endptr( char *p )
{
	return xml_find_end( p, "mods" );
}

static int
modsin_readf( FILE *fp, char *buf, int bufsize, int *bufpos, str *line, str *reference, int *fcharset )
{
	str tmp;
	int m, file_charset = CHARSET_UNKNOWN;
	char *startptr = NULL, *endptr = NULL;

	str_init( &tmp );

	do {
		if ( line->data ) str_strcat( &tmp, line );
		if ( str_has_value( &tmp ) ) {
			m = xml_getencoding( &tmp );
			if ( m!=CHARSET_UNKNOWN ) file_charset = m;
			startptr = modsin_startptr( tmp.data );
			endptr = modsin_endptr( tmp.data );
		} else startptr = endptr = NULL;
		str_empty( line );
		if ( startptr && endptr ) {
			str_segcpy( reference, startptr, endptr );
			str_strcpyc( line, endptr );
		}
	} while ( !endptr && str_fget( fp, buf, bufsize, bufpos, line ) );

	str_free( &tmp );
	*fcharset = file_charset;
	return ( reference->len > 0 );
}

