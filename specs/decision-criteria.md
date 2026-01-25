# Decision Criteria

## The Question

How do we build our VRP solver? Four options:

| Option | Description |
|--------|-------------|
| **A. Depend** | Use `vrp-pragmatic` as a cargo dependency |
| **B. Fork** | Fork `reinterpretcat/vrp` and maintain our own version |
| **C. Extract** | Copy relevant algorithms, build simpler solver |
| **D. Scratch** | Build from scratch using general metaheuristics |

---

## Evaluation Criteria

### 1. Correctness (Weight: Critical)

Does it produce valid solutions that respect all constraints?

| Option | Assessment | Notes |
|--------|------------|-------|
| A. Depend | ⚠️ Unknown | Open bugs suggest constraint violations possible |
| B. Fork | ⚠️ Unknown | Same bugs, but we can fix them |
| C. Extract | ✅ Likely | Simpler code = easier to verify |
| D. Scratch | ✅ Likely | We control everything |

**Research needed**: Run vrp-cli on test problems and verify constraints.

### 2. Solution Quality (Weight: High)

Are the routes reasonably efficient?

| Option | Assessment | Notes |
|--------|------------|-------|
| A. Depend | ✅ Good | Years of tuning, sophisticated algorithms |
| B. Fork | ✅ Good | Same algorithms |
| C. Extract | ⚠️ Moderate | Simpler algorithms = potentially worse |
| D. Scratch | ⚠️ Unknown | Depends on our implementation |

**Research needed**: Benchmark different approaches on realistic problems.

### 3. Maintainability (Weight: High)

Can we fix bugs, add features, understand what's happening?

| Option | Assessment | Notes |
|--------|------------|-------|
| A. Depend | ❌ Poor | Stuck with upstream bugs, can't modify |
| B. Fork | ⚠️ Moderate | 10k+ lines to understand and maintain |
| C. Extract | ✅ Good | 2-3k lines, we wrote it, we understand it |
| D. Scratch | ✅ Good | Same as extract |

### 4. Time to First Working Version (Weight: Medium)

How fast can we get something running?

| Option | Assessment | Notes |
|--------|------------|-------|
| A. Depend | ✅ Fast | Just write conversion layer |
| B. Fork | ⚠️ Medium | Need to understand codebase first |
| C. Extract | ⚠️ Medium | Need to read and adapt code |
| D. Scratch | ❌ Slow | Building from nothing |

### 5. Performance (Weight: Low for our scale)

At 5-10 drivers and 50-100 visits, performance is unlikely to be a bottleneck.

| Option | Assessment | Notes |
|--------|------------|-------|
| A. Depend | ✅ Good | Optimized implementation |
| B. Fork | ✅ Good | Same |
| C. Extract | ✅ Good | Simpler might even be faster for small problems |
| D. Scratch | ⚠️ Unknown | Depends on implementation |

---

## Current Assessment (Updated)

### Research Completed

1. ✅ **Test vrp-cli on realistic problems**
   - Created `specs/test-problems/service-basic.json`
   - 10 visits, 3 drivers, pinned drivers, time windows, breaks
   - **All constraints respected** in test
   - Solved in <1 second

2. ✅ **Read vrp-core source**
   - Algorithms are tightly coupled to internal types (`InsertionContext`, `RouteContext`, etc.)
   - Local search operators are ~200-300 lines each
   - Skills constraint is clean (~113 lines, good reference)
   - Full extraction would require bringing over domain types

3. ✅ **Estimate extraction effort**
   - Cannot use their data structures without also taking the whole framework
   - Would need our own domain types regardless
   - Estimated ~2,200 lines for a simplified solver with similar capabilities
   - vs 35,000 lines in vrp-core + rosomaxa + vrp-pragmatic

### Revised Assessment

| Option | Feasibility | Risk | Notes |
|--------|------------|------|-------|
| A. Depend | ✅ Works | ⚠️ Medium | Test passed, but 44 open issues |
| B. Fork | ✅ Possible | ⚠️ High | 35k lines to maintain |
| C. Extract | ✅ Possible | Low | ~2k lines, algorithms are well-documented |
| D. Scratch | ✅ Possible | Low | Same effort as C |

### Recommendation

**Option C (Extract/Inspire)** is recommended:

1. Build our own domain types (simple, ~500 lines)
2. Implement nearest neighbor construction (~200 lines)
3. Implement cheapest insertion heuristic (~400 lines)
4. Add local search: 2-opt and relocate (~500 lines)
5. Add simple ruin & recreate (~300 lines)
6. Constraint checking for skills, time windows (~300 lines)

This gives us:
- **Full control** - we understand every line
- **Debuggable** - no black box
- **Maintainable** - 2k lines vs 35k
- **Extensible** - add constraints as needed
- **Proven algorithms** - based on well-known VRP techniques

**vrp-core remains valuable as reference** - keep it in `vendor/` for algorithm inspiration.

---

## Remaining Research (Optional)

- [ ] Try SolverForge for constraint programming approach
- [ ] Test vrp-cli with 50-100 jobs to verify scale
- [ ] Research academic papers on VRP heuristics

---

## Decision Point

**Ready to decide.** Research tasks 1-3 are complete. The recommendation is Option C.

Next step: Design our domain types and basic solver architecture.
