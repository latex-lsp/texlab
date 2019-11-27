/*
 * xml.h
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef XML_H
#define XML_H

#include "slist.h"
#include "str.h"

typedef struct xml {
	str tag;
	str value;
	slist attributes;
	slist attribute_values;
	struct xml *down;
	struct xml *next;
} xml;

void   xml_init                 ( xml *node );
void   xml_free                 ( xml *node );
int    xml_has_value            ( xml *node );
str *  xml_value                ( xml *node );
char * xml_value_cstr           ( xml *node );
str *  xml_tag                  ( xml *node );
char * xml_tag_cstr             ( xml *node );
int    xml_tag_matches          ( xml *node, const char *tag );
int    xml_tag_matches_has_value( xml *node, const char *tag );
str *  xml_attribute            ( xml *node, const char *attribute );
char * xml_find_start           ( char *buffer, char *tag );
char * xml_find_end             ( char *buffer, char *tag );
int    xml_tag_has_attribute    ( xml *node, const char *tag, const char *attribute, const char *attribute_value );
int    xml_has_attribute        ( xml *node, const char *attribute, const char *attribute_value );
const char * xml_parse                ( const char *p, xml *onode );

extern char * xml_pns; /* global Namespace */

#endif

