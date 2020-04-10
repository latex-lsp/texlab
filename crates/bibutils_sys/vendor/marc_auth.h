/*
 * marc_auth.h
 *
 * Recognize the MARC authority vocabulary for genre and resource.
 *
 * Copyright (c) Chris Putnam 2008-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef MARC_AUTH_H
#define MARC_AUTH_H

int marc_findgenre( const char *query );
int is_marc_genre( const char *query );
int marc_findresource( const char *query );
int is_marc_resource( const char *query );
char *marc_convertrole( const char *query );

#endif
