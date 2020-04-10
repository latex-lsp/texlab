/* strsearch.c
 *
 * Copyright (c) Chris Putnam 1995-2019
 *
 * Source code released under the GPL version 2
 *
 * strsearch() locates a case-independent substring
 *    e.g. a case-independent version of strstr()
 *
 * returns pointer to first occurrence of substring needle in
 * the string haystack when found, NULL if not found
 *
 * '\0' characters terminating strings are not compared
 *
 * strsearch returns haystack when needle is empty as per strstr()
 * conventions
 *
 */
#include <stdio.h>
#include <ctype.h>
#include "strsearch.h"

char *strsearch (const char *haystack, const char *needle)
{
	char *returnptr=NULL;
	unsigned long pos=0;

	if ( !(*needle) ) returnptr = (char *) haystack;

	while (*(haystack+pos) && returnptr==NULL) {
		if ( toupper((unsigned char)*(haystack+pos)) == toupper((unsigned char)*(needle+pos)) )
			pos++;
		else {
			pos = 0;
			haystack++;
		}
		if ( ! (*(needle+pos)) ) returnptr = (char *) haystack;
	}
	return returnptr;
}

