## Compiling

To achieve a compilation of trec_eval for Windows, you will need Cygwin installed.

Download and install the [Cygwin](https://www.cygwin.com/) platform. You will need make and gcc installed by Cygwin - we would recomment selecting to install the Develop category.

Then, to compile trec_eval, open a Cygwin instance, navigate using cd to the directory of the trec_eval source, and type make.

## Running

The resulting trec_eval.exe should be usable on any machine without Cygwin installed, as long as the cygwin1.dll is available. For instance, place cygwin1.dll in the same directory as trec_eval.exe.
