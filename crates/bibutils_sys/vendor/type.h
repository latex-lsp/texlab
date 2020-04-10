/*
 * type.h
 *
 * Copyright (c) Chris Putnam 2019
 *
 * Source code released under the GPL version 2
 */
#ifndef TYPE_H
#define TYPE_H

#include <stdio.h>
#include <stdlib.h>
#include "fields.h"

#define TYPE_FROM_GENRE    (0)
#define TYPE_FROM_RESOURCE (1)
#define TYPE_FROM_ISSUANCE (2)

typedef struct match_type {
        char *name;
        int type;
        int level;
} match_type;

int type_from_mods_hints( fields *in, int mode, match_type matches[], int nmatches, int type_unknown );

#endif
