/*
 * bibutils.c
 *
 * Copyright (c) Chris Putnam 2005-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include <stdio.h>
#include <stdlib.h>
#include "bibutils.h"
#include "bibformats.h"

int
bibl_initparams( param *p, int readmode, int writemode, char *progname )
{
	int status;

	switch ( readmode ) {
	case BIBL_BIBTEXIN:     status = bibtexin_initparams  ( p, progname ); break;
	case BIBL_BIBLATEXIN:   status = biblatexin_initparams( p, progname ); break;
	case BIBL_COPACIN:      status = copacin_initparams   ( p, progname ); break;
	case BIBL_EBIIN:        status = ebiin_initparams     ( p, progname ); break;
	case BIBL_ENDNOTEIN:    status = endin_initparams     ( p, progname ); break;
	case BIBL_ENDNOTEXMLIN: status = endxmlin_initparams  ( p, progname ); break;
	case BIBL_MEDLINEIN:    status = medin_initparams     ( p, progname ); break;
	case BIBL_MODSIN:       status = modsin_initparams    ( p, progname ); break;
	case BIBL_NBIBIN:       status = nbibin_initparams    ( p, progname ); break;
	case BIBL_RISIN:        status = risin_initparams     ( p, progname ); break;
	case BIBL_WORDIN:       status = wordin_initparams    ( p, progname ); break;
	default:                status = BIBL_ERR_BADINPUT;
	}

	if ( status!=BIBL_OK ) return status;

	switch ( writemode ) {
	case BIBL_ADSABSOUT:   status = adsout_initparams   ( p, progname ); break;
	case BIBL_BIBTEXOUT:   status = bibtexout_initparams( p, progname ); break;
	case BIBL_ENDNOTEOUT:  status = endout_initparams   ( p, progname ); break;
	case BIBL_ISIOUT:      status = isiout_initparams   ( p, progname ); break;
	case BIBL_MODSOUT:     status = modsout_initparams  ( p, progname ); break;
	case BIBL_NBIBOUT:     status = nbibout_initparams  ( p, progname ); break;
	case BIBL_RISOUT:      status = risout_initparams   ( p, progname ); break;
	case BIBL_WORD2007OUT: status = wordout_initparams  ( p, progname ); break;
	default:               status = BIBL_ERR_BADINPUT;
	}

	return status;
}
