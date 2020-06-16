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

#if defined(_WIN32) || defined(_WIN64)
#ifndef _UNISTD_H
#define _UNISTD_H    1


#define index(a,b) strchr(a,b)

/* This is intended as a drop-in replacement for unistd.h on Windows.
 * Please add functionality as neeeded.
 * https://stackoverflow.com/a/826027/1202830
 */

#include <stdlib.h>
#include <io.h>
#include <process.h> /* for getpid() and the exec..() family */
#include <direct.h> /* for _getcwd() and _chdir() */

#define srandom srand
#define random rand

/* Values for the second argument to access.
   These may be OR'd together.  */
#define R_OK    4       /* Test for read permission.  */
#define W_OK    2       /* Test for write permission.  */
//#define   X_OK    1       /* execute permission - unsupported in windows*/
#define F_OK    0       /* Test for existence.  */

#define access _access
#define dup2 _dup2
#define execve _execve
#define ftruncate _chsize
#define unlink _unlink
#define fileno _fileno
#define getcwd _getcwd
#define chdir _chdir
#define isatty _isatty
#define lseek _lseek
/* read, write, and close are NOT being #defined here, because while there are file handle specific versions for Windows, they probably don't work for sockets. You need to look at your app and consider whether to call e.g. closesocket(). */

#ifdef _WIN64
#define ssize_t __int64
#else
#define ssize_t long
#endif

#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2
/* should be in some equivalent to <sys/types.h> */
typedef __int8            int8_t;
typedef __int16           int16_t; 
typedef __int32           int32_t;
typedef __int64           int64_t;
typedef unsigned __int8   uint8_t;
typedef unsigned __int16  uint16_t;
typedef unsigned __int32  uint32_t;
typedef unsigned __int64  uint64_t;

#endif /* unistd.h  */
#else
#include <unistd.h>
#endif
#include <limits.h>
#include <ctype.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <math.h>
#include <memory.h>
#include <sys/stat.h>
#if defined(_WIN32) || defined(_WIN64)
#include "mman.h"
#include "ya_getopt.h"
#else
#include <sys/mman.h>
#include <getopt.h>
#endif

/* see http://stackoverflow.com/questions/33058014/trec-eval-make-error-using-cygwin/34927338 */
#ifdef __CYGWIN__
#undef log2
#endif

/* For time being, define Berkeley constructs in terms of SVR4 constructs*/
#ifndef bzero
#define bzero(dest,len)      memset(dest,'\0',len)
#endif
#ifndef bcopy
#define bcopy(source,dest,len)   memcpy(dest,source,len)
#endif
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

#endif /* SYSFUNCH */
