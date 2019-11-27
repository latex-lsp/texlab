/*
 * xml.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "is_ws.h"
#include "strsearch.h"
#include "xml.h"

char *xml_pns = NULL;

void
xml_init( xml *node )
{
	str_init( &(node->tag) );
	str_init( &(node->value) );
	slist_init( &(node->attributes) );
	slist_init( &(node->attribute_values) );
	node->down = NULL;
	node->next = NULL;
}

static xml *
xml_new( void )
{
	xml *node = ( xml * ) malloc( sizeof( xml ) );
	if ( node ) xml_init( node );
	return node;
}

static void
xml_delete( xml *node )
{
	xml_free( node );
	free( node );
}

void
xml_free( xml *node )
{
	str_free( &(node->tag) );
	str_free( &(node->value) );
	slist_free( &(node->attributes) );
	slist_free( &(node->attribute_values) );
	if ( node->down ) xml_delete( node->down );
	if ( node->next ) xml_delete( node->next );
}

enum {
	XML_DESCRIPTOR,
	XML_COMMENT,
	XML_OPEN,
	XML_CLOSE,
	XML_OPENCLOSE
};

static int
xml_is_terminator( const char *p, int *type )
{
	if ( *p=='>' ) {
		return 1;
	} else if ( *p=='/' && *(p+1)=='>' ) {
		if ( *type==XML_OPENCLOSE ) return 1;
		else if ( *type==XML_OPEN ) {
			*type = XML_OPENCLOSE;
			return 1;
		}
	} else if ( *p=='?' && *(p+1)=='>' && *type==XML_DESCRIPTOR ) {
		return 1;
	} else if ( *p=='!' && *(p+1)=='>' && *type==XML_COMMENT ) {
		return 1;
	}
	return 0;
}

static int
xml_add_attribute( xml *node, char *attribute, char *attribute_value  )
{
	int status;

	if ( attribute )
		status = slist_addc( &(node->attributes), attribute );
	else
		status = slist_addc( &(node->attributes), "" );
	if ( status!=SLIST_OK ) return 0;

	if ( attribute_value )
		status = slist_addc( &(node->attribute_values), attribute_value );
	else
		status = slist_addc( &(node->attribute_values), "" );
	if ( status!=SLIST_OK ) {
		(void) slist_remove( &(node->attributes), node->attributes.n-1 );
		return 0;
	}
	return 1;
}

static const char *
xml_processattrib( const char *p, xml *node, int *type )
{
	char quote_character = '\"';
	int inquotes = 0;
	str aname, aval;

	str_init( &aname );
	str_init( &aval );

	while ( *p && !xml_is_terminator( p, type ) ) {

		/* get attribute name */
		while ( *p==' ' || *p=='\t' ) p++;
		while ( *p && !strchr( "= \t", *p ) && !xml_is_terminator( p, type ) ){
			str_addchar( &aname, *p );
			p++;
		}

		/* equals sign */
		while ( *p==' ' || *p=='\t' ) p++;
		if ( *p=='=' ) p++;
		while ( *p==' ' || *p=='\t' ) p++;

		/* get attribute value */
		if ( *p=='\"' || *p=='\'' ) {
			if ( *p=='\'' ) quote_character = *p;
			inquotes=1;
			p++;
		}
		while ( *p && ((!xml_is_terminator(p,type) && !strchr("= \t", *p ))||inquotes)){
			if ( *p==quote_character ) inquotes=0;
			else str_addchar( &aval, *p );
			p++;
		}
		if ( str_has_value( &aname ) ) {
			xml_add_attribute( node, str_cstr( &aname ), str_cstr( &aval ) );
		}

		str_empty( &aname );
		str_empty( &aval );
	}

	str_free( &aname );
	str_free( &aval );

	return p;
}

/*
 * xml_processtag
 *
 *                        start right after '<'
 *                        *
 *      XML_COMMENT      <!-- ....  -->
 * 	XML_DESCRIPTOR   <?.....>
 * 	XML_OPEN         <A>
 * 	XML_CLOSE        </A>
 * 	XML_OPENCLOSE    <A/>
 */
static const char *
xml_processtag( const char *p, xml *node, int *type )
{
	str tag;

	str_init( &tag );

	if ( *p=='!' ) {
		*type = XML_COMMENT;
		while ( *p && *p!='>' ) p++;
	}
	else if ( *p=='?' ) {
		*type = XML_DESCRIPTOR;
		p++; /* skip '?' */
		while ( *p && !strchr( " \t", *p ) && !xml_is_terminator(p,type) )
			str_addchar( &tag, *p++ );
		if ( *p==' ' || *p=='\t' )
			p = xml_processattrib( p, node, type );
	}
	else if ( *p=='/' ) {
		*type = XML_CLOSE;
		while ( *p && !strchr( " \t", *p ) && !xml_is_terminator(p,type) )
			str_addchar( &tag, *p++ );
		if ( *p==' ' || *p=='\t' ) 
			p = xml_processattrib( p, node, type );
	}
	else {
		*type = XML_OPEN;
		while ( *p && !strchr( " \t", *p ) && !xml_is_terminator(p,type) )
			str_addchar( &tag, *p++ );
		if ( *p==' ' || *p=='\t' ) 
			p = xml_processattrib( p, node, type );
	}
	while ( *p && *p!='>' ) p++;
	if ( *p=='>' ) p++;

	str_strcpy( &(node->tag), &tag );

	str_free( &tag );

	return p;
}

static void
xml_appendnode( xml *onode, xml *nnode )
{
	if ( !onode->down ) onode->down = nnode;
	else {
		xml *p = onode->down;
		while ( p->next ) p = p->next;
		p->next = nnode;
	}
}

const char *
xml_parse( const char *p, xml *onode )
{
	int type, is_style = 0;
	xml *nnode;

	while ( *p ) {

		/* retain white space for <style> tags in endnote xml */
		if ( str_cstr( &(onode->tag) ) &&
			!strcasecmp( str_cstr( &(onode->tag) ),"style") ) is_style=1;

		while ( *p && *p!='<' ) {
			if ( onode->value.len>0 || is_style || !is_ws( *p ) )
				str_addchar( &(onode->value), *p );
			p++;
		}

		if ( *p=='<' ) {
			nnode = xml_new();
			p = xml_processtag( p+1, nnode, &type );
			if ( type==XML_OPEN || type==XML_OPENCLOSE || type==XML_DESCRIPTOR ) {
				xml_appendnode( onode, nnode );
				if ( type==XML_OPEN )
					p = xml_parse( p, nnode );
			} else if ( type==XML_CLOSE ) {
				/*check to see if it's closing for this one*/
				xml_delete( nnode );
				goto out; /* assume it's right for now */
			} else {
				xml_delete( nnode );
			}
		}

	}
out:
	return p;
}

void
xml_draw( xml *node, int n )
{
	slist_index j;
	int i;

	if ( !node ) return;

	for ( i=0; i<n; ++i ) printf( "    " );

	printf("n=%d tag='%s' value='%s'\n", n, str_cstr( &(node->tag) ), str_cstr( &(node->value) ) );

	for ( j=0; j<node->attributes.n; ++j ) {
		for ( i=0; i<n; ++i ) printf( "    " );
		printf( "    attribute='%s' value='%s'\n",
			slist_cstr( &(node->attributes), j ),
			slist_cstr( &(node->attribute_values), j )
		);
	}

	if ( node->down ) xml_draw( node->down, n+1 );
	if ( node->next ) xml_draw( node->next, n );
}

char *
xml_find_start( char *buffer, char *tag )
{
	str starttag;
	char *p;

	str_initstrsc( &starttag, "<", tag, " ", NULL );

	p = strsearch( buffer, str_cstr( &starttag ) );
	if ( !p ) {
		starttag.data[ starttag.len-1 ] = '>';
		p = strsearch( buffer, str_cstr( &starttag ) );
	}

	str_free( &starttag );

	return p;
}

char *
xml_find_end( char *buffer, char *tag )
{
	str endtag;
	char *p;

	if ( xml_pns )
		str_initstrsc( &endtag, "</", xml_pns, ":", tag, ">", NULL );
	else
		str_initstrsc( &endtag, "</", tag, ">", NULL );

	p = strsearch( buffer, str_cstr( &endtag ) );
	if ( p && *p ) {
		if ( *p ) p++;  /* skip <random_tag></end> combo */
		while ( *p && *(p-1)!='>' ) p++;
	}

	str_free( &endtag );
	return p;
}

static int
xml_tag_matches_simple( xml* node, const char *tag )
{
	if ( node->tag.len!=strlen( tag ) ) return 0;
	if ( strcasecmp( str_cstr( &(node->tag) ), tag ) ) return 0;
	return 1;
}
static int
xml_tag_matches_pns( xml* node, const char *tag )
{
	int found = 0;
	str pnstag;

	str_initstrsc( &pnstag, xml_pns, ":", tag, NULL );
	if ( node->tag.len==pnstag.len &&
			!strcasecmp( str_cstr( &(node->tag) ), str_cstr( &pnstag ) ) )
		found = 1;
	str_free( &pnstag );

	return found;
}
int
xml_tag_matches( xml *node, const char *tag )
{
	if ( xml_pns ) return xml_tag_matches_pns   ( node, tag );
	else           return xml_tag_matches_simple( node, tag );
}

int
xml_tag_matches_has_value( xml *node, const char *tag )
{
	if ( xml_tag_matches( node, tag ) && xml_has_value( node ) ) return 1;
	return 0;
}

int
xml_has_attribute( xml *node, const char *attribute, const char *attribute_value )
{
	slist_index i;
	char *a, *v;

	for ( i=0; i<node->attributes.n; ++i ) {
		a = slist_cstr( &(node->attributes), i );
		v = slist_cstr( &(node->attribute_values), i );
		if ( !a || !v ) continue;
		if ( !strcasecmp( a, attribute ) && !strcasecmp( v, attribute_value ) )
			return 1;
	}

	return 0;
}

int
xml_tag_has_attribute( xml *node, const char *tag, const char *attribute, const char *attribute_value )
{
	if ( !xml_tag_matches( node, tag ) ) return 0;
	return xml_has_attribute( node, attribute, attribute_value );
}

str *
xml_attribute( xml *node, const char *attribute )
{
	slist_index n;

	n = slist_findc( &(node->attributes), attribute );
	if ( slist_wasnotfound( &(node->attributes), n ) ) return NULL;
	else return slist_str( &(node->attribute_values), n );
}

int
xml_has_value( xml *node )
{
	if ( node && str_has_value( &(node->value) ) ) return 1;
	return 0;
}

str *
xml_tag( xml *node )
{
	return &(node->tag);
}

char *
xml_tag_cstr( xml *node )
{
	return str_cstr( &(node->tag) );
}

str *
xml_value( xml *node )
{
	return &(node->value);
}

char *
xml_value_cstr( xml *node )
{
	return str_cstr( &(node->value) );
}
