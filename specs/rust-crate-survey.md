# Rust VRP Crate Survey

## Summary

There is essentially **one mature VRP-specific ecosystem** in pure Rust: `reinterpretcat/vrp`. Everything else is either forks of it, general-purpose optimization libraries, or too immature.

---

## VRP-Specific Crates

### reinterpretcat/vrp (Primary Candidate)

**Repository**: https://github.com/reinterpretcat/vrp
**Stars**: 469
**Last Release**: v1.25.0 (November 2024)
**License**: Apache-2.0

An ecosystem of crates:
- `vrp-core` - Core metaheuristic algorithms
- `vrp-pragmatic` - JSON-based problem definition for real-world scenarios
- `vrp-cli` - Command line interface
- `vrp-scientific` - Academic benchmark formats (Solomon, etc.)

**Supported VRP Variants**:
- Capacitated VRP (CVRP)
- VRP with Time Windows (VRPTW)
- VRP with Pickup and Delivery (VRPPD)
- Heterogeneous Fleet VRP (HFVRP)
- Multi-Depot VRP (MDVRP)
- Multi-Trip VRP (MTVRP)
- Open VRP (OVRP)
- Skill VRP (SVRP)
- Time-Dependent VRP (TDVRP)

**Constraints relevant to us**:
- Time windows ✅
- Skills (can map to "pinned driver") ✅
- Breaks ✅ (but buggy - see review)
- Multi-depot ✅
- Priorities ✅
- Job relations ✅
- Capacities ✅
- Vehicle shifts ✅

**See**: [vrp-pragmatic-review.md](vrp-pragmatic-review.md) for critical analysis

### rbilgil/rust-vrp

**Repository**: https://github.com/rbilgil/rust-vrp

This is just a **fork of reinterpretcat/vrp**. Not a separate project.

---

## General Optimization Crates

These are not VRP-specific but could be used as building blocks.

### SolverForge

**Repository**: https://github.com/SolverForge/solverforge
**Stars**: 7
**Created**: January 2026
**License**: Apache-2.0

A constraint programming framework (like OptaPlanner for Rust). Claims to solve "Vehicle Routing, Employee Scheduling, Bin Packing."

**Approach**: Declarative constraint API + metaheuristics (Hill Climbing, Simulated Annealing, Tabu Search, Late Acceptance)

**Assessment**:
- Very new (3 weeks old as of this writing)
- No VRP-specific modules - would need to build constraints ourselves
- Interesting architecture (zero-allocation, compile-time type resolution)
- Too immature to rely on, but worth watching

### good_lp

**Repository**: https://github.com/rust-or/good_lp

Linear/Mixed Integer Programming abstraction with multiple solver backends:
- Clarabel (pure Rust, LP only)
- HiGHS (C++, MIT, MIP)
- Microlp (pure Rust, slow)
- CBC, CPLEX, Gurobi (external)

**Assessment**:
- VRP can be formulated as MIP, but it's complex and doesn't scale well
- Better for small problems or as a component (e.g., assignment subproblems)
- Not the right tool for our main solver

### metaheuristics-nature

**Repository**: https://lib.rs/crates/metaheuristics-nature

Generic metaheuristic algorithms: genetic algorithms, particle swarm, differential evolution, etc.

**Assessment**:
- Would need to build all VRP logic ourselves
- Useful if we go the "build from scratch" route
- Provides the optimization loop, we provide the problem encoding

### optimization_engine (OpEn)

Nonconvex optimization for robotics. Not relevant for VRP.

---

## Research Progress

- [x] Clone reinterpretcat/vrp and explore the code structure
  - Cloned to `vendor/vrp/`
  - vrp-core: ~20,000 lines
  - rosomaxa: ~7,000 lines
  - vrp-pragmatic: ~8,000 lines
  - Total: ~35,000 lines of pure Rust

- [x] Run vrp-cli on a sample problem to see output quality
  - Created `specs/test-problems/service-basic.json`
  - 10 visits, 3 drivers, pinned drivers, time windows, breaks
  - **All constraints respected**
  - Solved in <1 second

- [x] Read vrp-core source to assess complexity and extractability
  - Algorithms are sophisticated but coupled to internal types
  - Skills constraint is clean (~113 lines)
  - Local search operators are ~200-300 lines each
  - Extraction possible but would need our own domain types

- [ ] Try SolverForge on a simple scheduling problem
- [ ] Look for academic papers on VRP heuristics we could implement ourselves
