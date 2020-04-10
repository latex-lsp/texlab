/*
 * title.h
 *
 * process titles into title/subtitle pairs for MODS
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL verison 2
 *
 */
#ifndef TITLE_H
#define TITLE_H

#include "str.h"
#include "fields.h"

int  title_process( fields *info, const char *tag, const char *value, int level, unsigned char nosplittitle );
void title_combine( str *fullttl, str *mainttl, str *subttl );

#endif
