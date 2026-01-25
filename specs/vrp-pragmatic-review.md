# Critical Review: vrp-pragmatic / reinterpretcat/vrp

## Overview

This is the only mature, VRP-specific pure-Rust solution. It deserves serious scrutiny before we commit to using it, forking it, or extracting from it.

**Confirmed: 100% pure Rust** - no C dependencies or FFI.

---

## Test Results

We ran vrp-cli on a realistic field-service routing problem with 10 visits, 3 drivers, pinned visits, and time windows. Results:

| Constraint | Result |
|------------|--------|
| `visit-2-pinned-to-alice` → alice-truck | ✅ Respected |
| `visit-6-pinned-to-bob` → bob-truck | ✅ Respected |
| Time window 10:00-11:00 | ✅ Arrived at edge of window |
| Time window 14:00-15:00 | ✅ Served within window |
| Bob's break (12:00-13:00) | ✅ Taken at 12:02-12:32 |
| Alice's break | ⚠️ Skipped (finished before break window) |

**Solution quality**: Reasonable. Used 2 of 3 drivers, total ~26km, ~7 hours.

**Performance**: Solved in <1 second (10 jobs, 3 vehicles).

---

## Red Flags

### 1. Maintainer Disclaimer

From the README:
> "Permanently experimental state, very limited support"

This is a personal project. The author is clear that support is limited.

### 2. Open Bugs Affecting Core Features

**44 open issues** as of January 2026. Key problems:

| Issue | Description | Impact |
|-------|-------------|--------|
| #168 | Missing break activities | Breaks don't appear in solution |
| #165 | ComponentRange error with breaks | Crashes |
| #138 | Solutions violate constraints | **Critical** - solver returns infeasible solutions |
| #136 | Non-repeatable randomization | Can't reproduce results |
| #145 | Time window violations with breaks | Constraints not honored |
| #131 | Capacity constraint issues | Core feature buggy |
| #130 | No waiting time cost | Can't penalize early arrivals |

### 3. Maintenance Responsiveness

Maintainer response on issue #144:
> "Currently, I'm not working actively on bug fixing, maybe will find some time later."

To be fair, he did eventually fix that specific issue, but the general stance is concerning.

### 4. Limited Adoption

- ~1,244 downloads in last 90 days (~14/day)
- Only 1 dependent crate on crates.io (likely just vrp-cli itself)
- Single author (Ilya Builuk / reinterpretcat)
- No known production users found
- 469 stars, but appears to be a personal/academic project, not industry-adopted

---

## Codebase Size

| Crate | Lines of Rust |
|-------|---------------|
| vrp-core | ~20,000 |
| rosomaxa (metaheuristics) | ~7,000 |
| vrp-pragmatic (JSON format) | ~8,000 |
| **Total** | **~35,000** |

This is substantial. Understanding and maintaining this codebase would be a significant investment.

---

## Architecture Analysis

### Construction Heuristics (`vrp-core/src/construction/`)

The solver uses sophisticated algorithms:

**Construction Phase** (`heuristics/`):
- Insertion evaluators - calculate cost of inserting a job at each position
- Route/job selectors - choose which jobs and routes to consider
- Result selectors - pick best insertion from candidates

**Features/Constraints** (`features/`):
- `skills.rs` - 113 lines, **clean and self-contained** ✅
- `breaks.rs` - handles driver breaks
- `capacity.rs` - vehicle capacity constraints
- `transport.rs` - time/distance constraints
- `locked_jobs.rs` - pinned/locked visits

### Metaheuristics (`vrp-core/src/solver/search/`)

**Local Search** (`local/`):
- `exchange_inter_route.rs` - swap jobs between routes (~240 lines)
- `exchange_intra_route.rs` - reorder jobs within a route
- `exchange_sequence.rs` - move sequences of jobs
- `exchange_swap_star.rs` - advanced swap operator

**Ruin and Recreate** (`ruin/`):
- `random_job_removal.rs` - simple, ~45 lines
- `worst_jobs_removal.rs` - remove expensive jobs, ~132 lines
- `neighbour_removal.rs` - remove nearby jobs, ~45 lines
- `cluster_removal.rs` - remove job clusters, ~74 lines
- `adjusted_string_removal.rs` - sophisticated, ~194 lines

### Key Insight: Coupled to Internal Types

The algorithms are **tightly coupled** to vrp-core's internal types:
- `InsertionContext` - central state object
- `RouteContext` - per-route state
- `Job`, `Activity` - problem representation
- `Goal` - objective function abstraction

Extracting algorithms without these types would be difficult.

---

## Extractability Assessment

### Easy to Extract
- **Skills constraint** (`skills.rs`) - self-contained, 113 lines
- **Basic ruin operators** - simple logic, could reimplement

### Medium Difficulty
- **Insertion heuristic** - well-structured but depends on abstractions
- **Local search operators** - algorithm is clear, coupling is the issue

### Hard to Extract
- **Full constraint framework** - deeply integrated
- **Rosomaxa population algorithm** - complex, proprietary

### Estimated Effort for Simplified Solver

If building our own with "inspiration" from vrp-core:

| Component | Lines | Difficulty |
|-----------|-------|------------|
| Domain types (Job, Route, Solution) | ~500 | Easy |
| Nearest neighbor construction | ~200 | Easy |
| Simple insertion heuristic | ~400 | Medium |
| Local search (2-opt, relocate) | ~500 | Medium |
| Constraint checking | ~300 | Medium |
| Ruin & recreate (basic) | ~300 | Medium |
| **Total** | **~2,200** | |

vs vrp-core's 20,000 lines + 7,000 rosomaxa.

---

## JSON Format Assessment

The pragmatic format is reasonable:

**Pros:**
- Well-documented
- Supports all our constraints
- Clean separation of plan/fleet/objectives

**Cons:**
- Some quirks (break format is picky)
- Would need a conversion layer regardless

---

## Questions Still Open

1. **How do the reported bugs manifest?**
   - Our simple test passed, but we didn't stress breaks + time windows together

2. **How reproducible are solutions?**
   - Issue #136 claims non-repeatable randomization

3. **How does it scale?**
   - Need to test with 50-100 jobs

---

## Verdict

**vrp-pragmatic is usable but risky.**

- ✅ Works for simple cases
- ✅ Pure Rust, good algorithms
- ✅ JSON format is reasonable
- ⚠️ Maintenance situation is concerning
- ⚠️ Known bugs in breaks/constraints
- ⚠️ 35k lines to understand if we need to debug

**Recommendation:** Consider **Option C (Extract)** - build a simpler ~2k line solver taking inspiration from vrp-core's algorithms, but with our own clean implementation. This gives us:
- Full control and understanding
- Ability to fix bugs ourselves
- Simpler codebase for our scale
- Still benefit from proven algorithmic approaches
