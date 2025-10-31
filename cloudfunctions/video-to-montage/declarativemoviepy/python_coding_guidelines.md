# Python Coding Standards: One Item Per File

These standards enforce extreme modularity and clarity by mandating a **one item per file** rule and explicit import style. They adapt the Rust guidelines to Python’s ecosystem.

---

### 1. Granularity: One Logical Item Per File

* **Rule:** Each `.py` file must define exactly one primary logical Python item.
* **Definition of "Item":**

    * A single `def` (function), `class`, or `Constant`.
    * Supporting helpers should go into their own files unless trivially private (e.g. `_helper` inside the same file).
* **File Naming:**

    * Snake case, matching the item: `my_function.py`, `my_class.py`.
* **Packages/Modules:**

    * `__init__.py` files are exceptions — they should only re-export items or declare submodules, never define items directly.

---

### 2. File Preamble Documentation

* **Rule:** Every `.py` file must start with a module-level docstring.
* **Structure:**

    * **First line:** One concise sentence summarizing the purpose.
    * **Blank line.**
    * **3–5 lines expanding:** Context, usage, considerations.
    * **Revision History:** Keep a bullet-style log of dated changes (ISO-8601 timestamps + author/AI tag).

```python
"""
Calculates the weighted sum of a list of numbers with given weights.

Takes two sequences of equal length. Computes sum(value * weight).
Returns 0.0 for empty lists. Raises ValueError if lengths mismatch.

Revision History:
- 2025-04-13T13:37:01Z @AI: Refined internal comments.
- 2025-04-13T13:24:57Z @AI: Added revision history section.
"""
```

---

### 3. Import Style: Fully Qualified and Explicit

* **Rule:** Avoid wildcard imports and implicit imports.
* **Usage:** Always import explicitly with the full path.

    * Standard library: `import math`, `import collections.abc`.
    * Third-party: `import numpy as np` is allowed, but aliases must be consistent and documented.
    * Internal: `from package.module import my_function` — never relative imports like `from .module import ...`.
* **Rationale:** Clear origin of every identifier.

---

### 4. Functional Style Encouraged

* **Guideline:** Prefer pure functions, immutability, and comprehensions/generator expressions.
* **Example:**

```python
def calculate_weighted_sum(values, weights):
    if len(values) != len(weights):
        raise ValueError("Value and weight lists must have the same length.")
    return sum(v * w for v, w in zip(values, weights))
```

---

### 5. Function Length Limit

* **Rule:** Functions must not exceed 50 lines of code.
* **Measurement:** Excludes docstring, comments, blank lines.
* **Exceptions:**

    * Large but simple `match`-like `if/elif` chains.
    * Data structure literals.
    * Auto-generated code.
* **Exception Justification:** Must include a `# NOTE:` inline comment.

---

### 6. In-File Tests for Functions

* **Rule:** Unit tests for an item must be colocated in the same file.
* **Structure:** Define tests under a `if __name__ == "__main__":` or local `unittest`/`pytest` block.
* **Scope:** Cover success, edge, and failure cases.

```python
def calculate_weighted_sum(values, weights):
    if len(values) != len(weights):
        raise ValueError("Value and weight lists must have the same length.")
    if not values:
        return 0.0
    return sum(v * w for v, w in zip(values, weights))


# ----------------- Tests -----------------

if __name__ == "__main__":
    import math

    # Basic case
    assert calculate_weighted_sum([1.0, 2.0, 3.0], [0.5, 1.0, 2.0]) == 8.5

    # Empty input
    assert calculate_weighted_sum([], []) == 0.0

    # Mismatched lengths
    try:
        calculate_weighted_sum([1.0, 2.0], [0.5])
    except ValueError as e:
        assert str(e) == "Value and weight lists must have the same length."

    # Negative numbers
    assert math.isclose(calculate_weighted_sum([-1.0, 2.0], [3.0, -0.5]), -4.0)
```

---

### 7. Public Data

* **Rule:** Public class attributes and module-level constants are allowed.
* **Convention:** Constants in `UPPER_CASE`, all else in `snake_case`.

