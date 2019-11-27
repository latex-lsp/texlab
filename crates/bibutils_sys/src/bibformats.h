/*
 * bibformats.h
 *
 * Copyright (c) Chris Putnam 2007-2019
 *
 * Source code released under the GPL version 2
 *
 */
#ifndef BIBFORMATS_H
#define BIBFORMATS_H

#include "bibutils.h"

int adsout_initparams    ( param *pm, const char *progname );
int biblatexin_initparams( param *pm, const char *progname );
int bibtexin_initparams  ( param *pm, const char *progname );
int bibtexout_initparams ( param *pm, const char *progname );
int copacin_initparams   ( param *pm, const char *progname );
int ebiin_initparams     ( param *pm, const char *progname );
int endin_initparams     ( param *pm, const char *progname );
int endout_initparams    ( param *pm, const char *progname );
int endxmlin_initparams  ( param *pm, const char *progname );
int isiin_initparams     ( param *pm, const char *progname );
int isiout_initparams    ( param *pm, const char *progname );
int medin_initparams     ( param *pm, const char *progname );
int modsin_initparams    ( param *pm, const char *progname );
int modsout_initparams   ( param *pm, const char *progname );
int nbibin_initparams    ( param *pm, const char *progname );
int nbibout_initparams   ( param *pm, const char *progname );
int risin_initparams     ( param *pm, const char *progname );
int risout_initparams    ( param *pm, const char *progname );
int wordin_initparams    ( param *pm, const char *progname );
int wordout_initparams   ( param *pm, const char *progname );

#endif
