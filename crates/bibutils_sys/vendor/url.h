/*
 * url.h
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef URL_H
#define URL_H

#include "slist.h"
#include "fields.h"

int is_doi( char *s );
int is_uri_remote_scheme( char *p );
int is_embedded_link( char *s );

void doi_to_url( fields *info, int n, char *urltag, str *doi_url );
void pmid_to_url( fields *info, int n, char *urltag, str *pmid_url );
void pmc_to_url( fields *info, int n, char *urltag, str *pmid_url );
void arxiv_to_url( fields *info, int n, char *urltag, str *arxiv_url );
void jstor_to_url( fields *info, int n, char *urltag, str *jstor_url );
void mrnumber_to_url( fields *info, int n, char *urltag, str *jstor_url );

int urls_merge_and_add( fields *in, int lvl_in, fields *out, char *tag_out, int lvl_out, slist *types );
int urls_split_and_add( char *value_in, fields *out, int lvl_out );


#endif
