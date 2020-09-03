# Windows Support

trec_eval can be compiled natively for Windows, using Visual Studio build tools, or using Cygwin

## Compiling using Visual Studio build tools

See the [Github Workflow](.github/workflow/push.yml) for compilation.

A few additional includes are used:
 - `mman.h` - a mmap() implementation for Windows
 - `unistd.h` - dropin replacement for the Unix header file unistd.h
 - `ya_getopt.h` - BSD licensed getopt_log() implementation

The resulting .exe can be used natively on Windows without any further dependencies.

## Compiling on Cygwin

Download and install the [Cygwin](https://www.cygwin.com/) platform. You will need make and gcc installed by Cygwin. To achieve this, on top of the default Cygwin installation, it is recommended to install automake, make, gcc, cygwin-gcc and git from the Develop category, and permitting dependencies to be installed.

Then, to compile trec_eval, open a Cygwin Terminal, navigate using cd to the directory of the trec_eval source, and type make.

## Running on Cygwin

The resulting trec_eval.exe should be usable directly from the Cygwin Terminal.

The resulting trec_eval.exe should be usable on any machine without Cygwin installed, as long as the cygwin1.dll is available. For instance, place a copy the cygwin1.dll from Cygwin's /bin directory into the same directory as trec_eval.exe.
