# VRP Planner Specifications

This folder contains research and specifications for the vehicle routing planner.

## Documents

1. **[problem-statement.md](problem-statement.md)** - What we're building and why
2. **[rust-crate-survey.md](rust-crate-survey.md)** - Survey of existing Rust VRP crates
3. **[vrp-pragmatic-review.md](vrp-pragmatic-review.md)** - Critical review of the main candidate
4. **[decision-criteria.md](decision-criteria.md)** - How we'll decide: use, copy, inspire, or build
5. **[routing-planner/](routing-planner/README.md)** - Detailed solver specification and plans

## Status

**Phase**: Research

We need to make an informed decision about whether to:
- Use an existing crate as a dependency
- Fork/copy code from an existing crate
- Take algorithmic inspiration but write our own
- Build from scratch

This is a significant architectural decision that requires thorough research.
