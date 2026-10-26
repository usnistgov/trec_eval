"""
Ecosystem compatibility adapters for legacy and modern IR libraries.
For ir_measures usage, see `ir_measures`.

Includes:
- `trec_eval.compat.pytrec_eval`: Drop-in replacement for the `pytrec_eval` library.
"""

from trec_eval.compat import pytrec_eval

__all__ = [
    "pytrec_eval",
]
