/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/
#ifndef SYSFUNCH
#define SYSFUNCH
/* Declarations of major functions within standard C libraries */
/* Once all of the major systems get their act together (and I follow
   suit!), this file should just include system header files from 
   /usr/include.  Until then... */

#ifdef _WIN32
#include <windows.h>
#include <direct.h>
#include <stdlib.h>
#include <stdio.h>
#include "mman-win32/mman.h"
#include "ya_getopt/ya_getopt.h"
#include "strings_h/strings.h"
#else
#include <unistd.h>
#include <sys/mman.h>
#include <getopt.h>
#include <strings.h>
#endif
#include <limits.h>
#include <ctype.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <memory.h>
#include <sys/stat.h>


/* For time being, define Berkeley constructs in terms of SVR4 constructs*/
#define bzero(dest,len)      memset(dest,'\0',len)
#define bcopy(source,dest,len)   memcpy(dest,source,len)
#define srandom(seed)        srand(seed)
#define random()             rand()

/* ANSI should give us an offsetof suitable for the implementation;
 * otherwise, try a non-portable but commonly supported definition
 */
#ifdef __STDC__
#include <stddef.h>
#endif
#ifndef offsetof
#define offsetof(type, member) ((size_t) \
	((char *)&((type*)0)->member - (char *)(type *)0))
#endif

#endif                          /* SYSFUNCH */
