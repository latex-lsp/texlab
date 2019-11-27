/*
 * bu_auth.h
 *
 * Recognize added vocabulary for genre terms added by bibutils.
 *
 * Copyright (c) Chris Putnam 2017-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef BU_AUTH_H
#define BU_AUTH_H

int bu_findgenre( const char *query );
int is_bu_genre( const char *query );

#endif
