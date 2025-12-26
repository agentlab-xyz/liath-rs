"""
Liath: Programmable Memory for AI Agents

Liath enables agents to write Lua code to safely query their own memory.
The core operation is `execute()` - run agent-generated Lua with structured
error feedback for self-correction.

Example:
    >>> from liath import Liath
    >>> db = Liath("./data")
    >>>
    >>> # Agent writes Lua to query memory
    >>> result = db.execute('''
    ...     local results = search("memories", "coding preferences", 10)
    ...     return json.encode(results)
    ... ''')
    >>>
    >>> if result["success"]:
    ...     print(result["value"])
    ... else:
    ...     # Structured error helps LLM fix the code
    ...     print(result["error"]["suggestion"])
"""

from liath._liath import (
    # Main class
    Liath,
    # Result types
    ValidationResult,
    ValidationError,
    SearchResult,
    # Exceptions
    LiathError,
    LiathValidationError,
    LiathRuntimeError,
)

__version__ = "0.1.0"
__all__ = [
    "Liath",
    "ValidationResult",
    "ValidationError",
    "SearchResult",
    "LiathError",
    "LiathValidationError",
    "LiathRuntimeError",
]
