## 🧭 Code Structure Philosophy

This repository follows a **strict one-file-per-item structure**. Each file defines exactly one thing: a function, struct, enum, trait, or constant.

This isn't just a stylistic choice — it's a transitional step toward a deeper structural shift in how codebases are represented and maintained.

See also:

* [`rust_coding_guidelines.md`](./rust_coding_guidelines.md)
* [`typescript_frontend_guidelines.md`](./typescript_frontend_guidelines.md)

### 🤖 Toward AI-Native Codebases

This project treats code as a **graph**. Each file is a node; imports and references form edges. The long-term vision is that codebases will live in **databases**, not files — where every symbol is a uniquely addressable, queryable entity.

Switching to one-file-per-item is a **practical intermediate step** toward that future. It simplifies the job of AI assistants, makes dependency graphs explicit, and increases the probability of correct edits, insertions, and reasoning — especially under constraints like limited context windows or tool-driven refactoring.

### 🧪 A Design Experiment

This is an experiment and a leap of faith.

We assume that code today is **mostly written and maintained by AI**, and that means the structure must serve the AI just as much as the human. Many projects break under complexity. This one doesn't — not because it’s simple, but because the complexity is structured, local, and tractable.

We're designing a codebase where **change is the default**, and AI tooling can reason about and navigate the system without global knowledge. We accept strictness and verbosity as tradeoffs for tractability, testability, and predictable evolution.
