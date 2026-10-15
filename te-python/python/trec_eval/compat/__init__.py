"""
Ecosystem compatibility adapters for legacy and modern IR libraries.

Includes:
- `trec_eval.compat.pytrec_eval`: Drop-in replacement for the `pytrec_eval` library.
- `trec_eval.compat.ir_measures`: High-performance evaluation provider for `ir_measures`.
"""

from trec_eval.compat import pytrec_eval
from trec_eval.compat import ir_measures

__all__ = [
    "pytrec_eval",
    "ir_measures",
]
