# VRP Solver Algorithm Documentation

## Executive Summary

The VRP solver implements a two-phase optimization approach: a **greedy construction heuristic** followed by **local search refinement**. The construction phase assigns visits to visitors one at a time, selecting the lowest-cost feasible insertion position across all routes. The local search phase then iteratively applies 2-opt (intra-route reversal) and relocate (inter-route movement) operators until no further improvement is found or iteration limits are reached. The solver respects hard constraints including time windows, visitor availability, capability requirements, and pinned assignments, while minimizing a cost function that combines travel time, target time deviation, and stability penalties.

---

## Part 1: Current Implementation

### Algorithm Overview

```
+------------------+     +------------------+     +------------------+
|   PREPROCESSING  | --> |   CONSTRUCTION   | --> |   LOCAL SEARCH   |
|      PHASE       |     |      PHASE       |     |      PHASE       |
+------------------+     +------------------+     +------------------+
        |                        |                        |
        v                        v                        v
  - Classify visits        Single-pass greedy      2-opt + relocate
  - Handle pinned visits   insertion heuristic     improvement
  - Build distance matrix  O(n^3 * m)              O(k * m^2 * n^2)
```

### Phase 1: Preprocessing

Before construction begins, the solver:

1. **Classifies visits by pin type**:
   - `Visitor` or `VisitorAndDate`: Pre-assigned to specific visitors
   - `Date`: Must be on this date, but visitor is flexible
   - `None`: Fully flexible

2. **Filters visits by date**: Visits pinned to a different date are marked unassigned with `WrongDate` reason.

3. **Initializes routes**: Creates one route per visitor, pre-populated with any pinned visits.

4. **Builds distance matrix**: Collects all unique locations and requests travel times from the matrix provider (OSRM or Haversine fallback).

### Phase 2: Construction (Greedy Insertion)

The construction phase processes visits one at a time:

```rust
for each unassigned visit:
    best_cost = infinity
    best_route = None
    best_position = None

    for each route (in parallel):
        if visitor lacks required capabilities:
            skip

        for each position in 0..=route.length:
            candidate = insert visit at position
            if feasible(candidate):
                cost = compute_schedule_cost(candidate)
                if cost < best_cost:
                    best_cost = cost
                    best_route = route
                    best_position = position

    if best_route found:
        insert visit at best_position in best_route
    else:
        mark visit as unassigned
```

**Key characteristics:**
- Single-pass: Each visit is placed once and not reconsidered during construction
- Greedy: Selects the minimum-cost feasible position at each step
- Parallel evaluation: Routes are evaluated in parallel using Rayon
- No lookahead: Does not consider how current placement affects future placements

### Phase 3: Local Search

After construction, the solver applies local search operators iteratively:

```rust
for iteration in 0..max_iterations:
    improved = false

    // Intra-route improvement
    for each route:
        if two_opt_improve(route):
            improved = true

    // Inter-route improvement
    if relocate_improve(routes):
        improved = true

    if not improved:
        break  // Local optimum reached
```

#### 2-opt Operator

Reverses a segment within a single route to reduce travel time:

```
Before:  A -> B -> C -> D -> E

2-opt(1, 3):
After:   A -> D -> C -> B -> E
              ^^^^^^^
              reversed
```

The operator tries all segment reversals and accepts the first improvement found (first-improvement strategy).

**Complexity**: O(n^2) per route, where n is the number of visits in the route.

#### Relocate Operator

Moves a single visit from one route to another (or to a different position in the same route):

```
Before:                    After:
Route 1: A -> B -> C       Route 1: A -> C
Route 2: D -> E            Route 2: D -> B -> E
```

The operator:
1. Tries moving each visit from each route
2. Evaluates all possible insertion positions in all routes
3. Respects pinned visitor constraints (won't move pinned visits between routes)
4. Accepts the first improvement found

**Complexity**: O(m^2 * n^2) per iteration, where m is the number of routes and n is the average visits per route.

### Constraint Handling

#### Hard Constraints (Must Be Satisfied)

| Constraint | How Enforced |
|------------|--------------|
| **Visitor Availability** | `compute_schedule` returns `None` if no valid window exists |
| **Time Windows** | Service must fit entirely within an availability window |
| **Committed Windows** | Visit must start within its committed time range |
| **Capabilities** | Visitor must have all required capabilities for the visit |
| **Pinned Visitor** | Visits pinned to a visitor cannot be reassigned |
| **Pinned Date** | Visits pinned to wrong date are immediately unassigned |

#### Soft Constraints (Penalized in Cost Function)

| Constraint | Penalty |
|------------|---------|
| **Target Time Deviation** | `\|actual_start - target_time\| * target_time_weight` |
| **Reassignment** | `reassignment_penalty` if assigned to different visitor than current |

### Cost Function

The total cost for a route is computed as:

```
route_cost = total_travel_time
           + sum(target_time_penalties)
           + sum(reassignment_penalties)

where:
  total_travel_time = sum of travel time between consecutive visits
  target_time_penalty = |start_time - target_time| * target_time_weight
  reassignment_penalty = reassignment_penalty if visitor changed
```

**Objective**: Minimize the sum of route costs across all routes, subject to:
1. All hard constraints satisfied
2. Minimize unassigned visits (implicit - unassigned visits have infinite cost)

### Complexity Analysis

#### Time Complexity

| Phase | Complexity | Explanation |
|-------|------------|-------------|
| **Matrix Construction** | O(L^2) | L = unique locations; depends on provider |
| **Preprocessing** | O(n + m) | n = visits, m = visitors |
| **Construction** | O(n^3 * m) | For each of n visits, evaluate O(n) positions in m routes, each evaluation is O(n) |
| **Local Search (per iteration)** | O(m * n^2 + m^2 * n^2) | 2-opt on m routes + relocate across m^2 route pairs |
| **Local Search (total)** | O(k * m^2 * n^2) | k = number of iterations (max: local_search_iterations) |

**Overall**: O(n^3 * m + k * m^2 * n^2) where typically k << n.

#### Space Complexity

| Component | Complexity | Notes |
|-----------|------------|-------|
| Distance Matrix | O(L^2) | L = unique locations |
| Coordinate Index | O(L) | HashMap for fast lookups |
| Routes | O(n + m) | Visit references distributed across m routes |
| Candidate Solutions | O(n) | Temporary during evaluation |

### Configuration Options

```rust
pub struct SolveOptions {
    /// Weight for target time deviation penalty (per second).
    /// Default: 1
    pub target_time_weight: i32,

    /// Weight for reassigning a visit to a different visitor.
    /// Default: 300 (~5 minutes equivalent)
    pub reassignment_penalty: i32,

    /// Maximum iterations for local search improvement.
    /// Default: 100
    pub local_search_iterations: usize,
}
```

| Option | Default | Effect |
|--------|---------|--------|
| `target_time_weight` | 1 | Higher values prioritize meeting target times over minimizing travel |
| `reassignment_penalty` | 300 | Higher values favor keeping visits with their current visitor |
| `local_search_iterations` | 100 | More iterations may find better solutions but increase runtime |

### Distance Matrix Providers

The solver supports pluggable distance matrix providers:

| Provider | Description | Use Case |
|----------|-------------|----------|
| `OsrmClient` | Real road network routing via OSRM | Production use with accurate travel times |
| `HaversineMatrix` | Great-circle distance approximation | Development/testing, fallback when OSRM unavailable |

**Haversine assumptions:**
- Default speed: 40 km/h
- Symmetric distances (A->B = B->A)
- Ignores actual road network

### Limitations

1. **Single-pass construction**: Visits are placed in input order. Earlier placements may prevent better overall solutions. No backtracking or regret-based insertion.

2. **First-improvement local search**: Accepts the first improving move rather than searching for the best improvement. May converge to worse local optima.

3. **Limited operator set**: Only 2-opt and relocate. Missing operators like Or-opt, 3-opt, exchange (swap between routes), and cross-exchange.

4. **No metaheuristic wrapper**: No simulated annealing, tabu search, or genetic algorithm to escape local optima.

5. **Greedy-only construction**: No randomization or diversification in initial solution building.

6. **No time-dependent routing**: Assumes static travel times. Cannot model rush-hour traffic.

7. **No multi-trip support**: Each visitor has exactly one route per day.

8. **Large problem scaling**: Construction is O(n^3*m), which becomes slow for hundreds of visits.

---

## Part 2: Roadmap for Improving Solver Sophistication

### Tier 1: Quick Wins (Low effort, moderate impact)

#### 1.1 Multiple Initial Solutions

**What it is**: Generate several initial solutions using different strategies and keep the best one (or all for population-based search later).

**Why it helps**: Different construction orderings find different local optima. The greedy nature of construction means the final solution depends heavily on the order visits are considered.

**Strategies to implement**:
- **Farthest-first insertion**: Start with visits farthest from depot
- **Nearest-neighbor**: Build routes by always adding the nearest unassigned visit
- **Randomized greedy**: Add random noise to insertion costs
- **Time-window ordered**: Process visits by earliest/latest deadline

**Implementation complexity**: Low
**Expected benefit**: 5-15% improvement in solution quality
**Dependencies**: None

---

#### 1.2 Regret Insertion

**What it is**: Instead of inserting the visit with minimum cost, insert the visit that would suffer most if not inserted now.

```
regret_k(visit) = cost_of_kth_best_position - cost_of_best_position

Select visit with maximum regret_k
```

**Why it helps**: Prevents "difficult" visits (those with few good options) from becoming unassignable. Particularly valuable when time windows are tight.

**Implementation complexity**: Low
**Expected benefit**: 5-10% fewer unassigned visits, 2-5% cost improvement
**Dependencies**: None

---

#### 1.3 Best-Improvement Local Search

**What it is**: Instead of accepting the first improvement found, evaluate all possible moves and apply the best one.

**Why it helps**: First-improvement can get trapped in poor local optima. Best-improvement explores more thoroughly per iteration.

**Trade-off**: Slower per iteration but may converge to better solutions with fewer iterations.

**Implementation complexity**: Low (change loop structure)
**Expected benefit**: 2-5% cost improvement
**Dependencies**: None

---

#### 1.4 Or-opt Operator

**What it is**: Move a sequence of 1-3 consecutive visits to a different position in the same route or another route.

```
Before:  A -> [B -> C] -> D -> E
After:   A -> D -> [B -> C] -> E
```

**Why it helps**: More powerful than single relocate; can move naturally clustered visits together.

**Implementation complexity**: Low
**Expected benefit**: 3-8% cost improvement when visits cluster geographically
**Dependencies**: None

---

#### 1.5 3-opt Operator

**What it is**: Generalization of 2-opt that reconnects three edges instead of two.

**Why it helps**: Can escape local optima that 2-opt cannot. Handles cases where reversing a single segment is insufficient.

**Implementation complexity**: Medium (8 reconnection cases)
**Expected benefit**: 2-5% cost improvement
**Dependencies**: None

---

### Tier 2: Moderate Improvements (Medium effort, good impact)

#### 2.1 Simulated Annealing Wrapper

**What it is**: Probabilistically accept worse solutions to escape local optima. Acceptance probability decreases over time (cooling schedule).

```
delta = new_cost - current_cost
if delta < 0:
    accept
else:
    accept with probability exp(-delta / temperature)

temperature *= cooling_rate  // e.g., 0.995
```

**Why it helps**: Allows escaping local optima. Well-studied, parameter-tunable, and effective for VRP.

**Implementation complexity**: Medium
**Expected benefit**: 10-20% improvement over pure local search
**Dependencies**: None, but works best with diverse operators (Tier 1)

---

#### 2.2 Tabu Search

**What it is**: Maintain a list of recently made moves and forbid reversing them for a number of iterations.

```
tabu_list = recent (visit, old_route, new_route) moves
if move in tabu_list and not aspiration_criterion:
    skip move
```

**Why it helps**: Prevents cycling between solutions. Forces exploration of new areas.

**Implementation complexity**: Medium
**Expected benefit**: 10-15% improvement, especially for constrained problems
**Dependencies**: Pair with aspiration criteria (accept tabu move if it improves best-known)

---

#### 2.3 Adaptive Large Neighborhood Search (ALNS)

**What it is**: Iteratively destroy and repair solutions using multiple operators. Adapt operator selection based on historical performance.

```
while not termination:
    select destroy_operator based on weights
    select repair_operator based on weights

    partial_solution = destroy(current_solution)
    new_solution = repair(partial_solution)

    if accept(new_solution):
        current_solution = new_solution

    update operator weights based on outcome
```

**Destroy operators**: Random removal, worst removal, related removal (geographically close), cluster removal

**Repair operators**: Greedy insertion, regret insertion, random insertion

**Why it helps**: Different problem regions benefit from different operators. Adaptation learns what works.

**Implementation complexity**: Medium-High
**Expected benefit**: 15-25% improvement
**Dependencies**: Multiple destroy/repair operators, acceptance criterion (SA or threshold)

---

#### 2.4 Parallel Local Search

**What it is**: Run multiple local search threads with different starting solutions or random seeds. Share best solutions periodically.

**Why it helps**: Explores multiple regions of solution space simultaneously. Good for utilizing multi-core systems.

**Implementation complexity**: Medium (Rayon already available)
**Expected benefit**: Near-linear speedup with cores; better solution quality from diversity
**Dependencies**: Multiple initial solutions (Tier 1.1)

---

#### 2.5 Exchange (SWAP) Operator

**What it is**: Swap a visit from one route with a visit from another route.

```
Before:                    After:
Route 1: A -> B -> C       Route 1: A -> D -> C
Route 2: D -> E            Route 2: B -> E
```

**Why it helps**: Can rebalance routes and fix poor initial assignments. Single move achieves what would take multiple relocates.

**Implementation complexity**: Medium
**Expected benefit**: 3-8% cost improvement, better load balancing
**Dependencies**: None

---

### Tier 3: Advanced Features (High effort, high impact)

#### 3.1 Genetic Algorithm / Evolutionary Approach

**What it is**: Maintain a population of solutions. Create new solutions through crossover (combining parents) and mutation (local changes). Select survivors based on fitness.

```
population = [initial_solutions]

while not termination:
    parents = select(population)  // tournament selection
    offspring = crossover(parents)  // route-based or order crossover
    offspring = mutate(offspring)   // local search
    population = survivor_selection(population + offspring)
```

**Why it helps**: Population diversity prevents premature convergence. Crossover can combine good route structures from different solutions.

**Crossover strategies for VRP**:
- **Route-based crossover**: Inherit complete routes from parents
- **Order crossover (OX)**: Preserve relative visit ordering
- **Edge assembly crossover**: Preserve good edges from both parents

**Implementation complexity**: High
**Expected benefit**: 20-30% improvement over local search alone
**Dependencies**: Multiple initial solutions, good mutation operator (local search)

---

#### 3.2 Population Management with Diversity

**What it is**: Maintain solution diversity by measuring solution distance and culling similar solutions.

**Solution distance metrics**:
- Visit-to-route mapping difference
- Edge set difference
- Objective vector difference (multi-objective)

**Why it helps**: Prevents population collapse to single solution type. Maintains exploration capability.

**Implementation complexity**: High
**Expected benefit**: Essential for genetic algorithm effectiveness
**Dependencies**: Genetic algorithm (Tier 3.1)

---

#### 3.3 Machine Learning for Operator Selection

**What it is**: Use reinforcement learning (e.g., Multi-Armed Bandit, Q-learning) to select which operator to apply.

**Features to consider**:
- Current solution characteristics (number of routes, utilization, slack)
- Problem characteristics (time window tightness, geographic distribution)
- Search phase (early exploration vs late intensification)

**Why it helps**: Learns problem-specific operator effectiveness. Adapts during search.

**Implementation complexity**: High
**Expected benefit**: 5-15% improvement over fixed operator selection
**Dependencies**: Multiple operators (Tier 1 & 2), sufficient training data

---

#### 3.4 Problem Decomposition

**What it is**: Divide large problems into smaller subproblems, solve independently, then merge.

**Decomposition strategies**:
- **Geographic clustering**: Group visits by location
- **Time-based**: Group visits by time windows
- **Route-based**: Optimize subsets of routes independently

```
clusters = cluster_visits(problem)
sub_solutions = parallel_solve(clusters)
solution = merge(sub_solutions)
solution = boundary_repair(solution)  // fix inter-cluster issues
```

**Why it helps**: Reduces O(n^3) to O(k * (n/k)^3). Essential for large instances.

**Implementation complexity**: High
**Expected benefit**: 10x+ speedup for large problems (500+ visits)
**Dependencies**: Good clustering, boundary repair mechanism

---

### Tier 4: Production Considerations

#### 4.1 Anytime Algorithm

**What it is**: Return the best solution found so far at any time. Support interruption without losing progress.

```
best_solution = initial_solution

while not timeout:
    new_solution = improve(current_solution)
    if new_solution better than best_solution:
        best_solution = new_solution
        report_progress(best_solution)

return best_solution
```

**Why it helps**: Production environments often have time constraints. Users want to see progress.

**Implementation complexity**: Low-Medium
**Expected benefit**: Essential for production use
**Dependencies**: None

---

#### 4.2 Time-Bounded Optimization

**What it is**: Accept a time budget and allocate effort appropriately.

**Strategies**:
- Fast initial solution + progressively expensive improvement
- Early termination when improvement rate drops
- Reserve time for final polish

**Implementation complexity**: Medium
**Expected benefit**: Consistent runtime, better resource utilization
**Dependencies**: Anytime algorithm (Tier 4.1)

---

#### 4.3 Solution Quality Metrics

**What it is**: Report solution characteristics beyond just cost.

**Metrics to track**:
- Total travel time vs theoretical minimum
- Time window utilization (slack)
- Route balance (coefficient of variation)
- Number of unassigned visits
- Constraint violation summary

**Why it helps**: Helps users understand solution trade-offs. Enables A/B testing of algorithm changes.

**Implementation complexity**: Low
**Expected benefit**: Better decision-making, debugging capability
**Dependencies**: None

---

#### 4.4 A/B Testing Framework

**What it is**: Infrastructure to compare algorithm variants on historical data.

**Components**:
- Problem instance storage
- Deterministic seeding
- Metric collection and comparison
- Statistical significance testing

**Why it helps**: Data-driven algorithm development. Prevents regressions.

**Implementation complexity**: Medium-High
**Expected benefit**: Essential for algorithm iteration
**Dependencies**: Solution quality metrics (Tier 4.3)

---

### Implementation Priority Recommendation

For immediate impact with minimal risk:

1. **Tier 1.2 (Regret Insertion)** - Reduces unassigned visits
2. **Tier 1.1 (Multiple Initial Solutions)** - Easy wins from diversity
3. **Tier 1.4 (Or-opt)** - Complements existing operators
4. **Tier 4.1 (Anytime)** - Production readiness

For significant quality improvement:

5. **Tier 2.1 (Simulated Annealing)** - Well-understood, effective
6. **Tier 2.3 (ALNS)** - State-of-the-art for VRP
7. **Tier 2.5 (Exchange)** - Essential operator for SA/ALNS

For scaling to larger problems:

8. **Tier 3.4 (Decomposition)** - Required for 500+ visits

---

## Part 3: References

### Core VRP References

- **Toth & Vigo (2014)**: "Vehicle Routing: Problems, Methods, and Applications" - Comprehensive textbook
- **Laporte (2009)**: "Fifty Years of Vehicle Routing" - Historical survey

### Metaheuristics

- **Kirkpatrick et al. (1983)**: "Optimization by Simulated Annealing" - Original SA paper
- **Glover (1986)**: "Future Paths for Integer Programming and Links to Artificial Intelligence" - Tabu search origins
- **Ropke & Pisinger (2006)**: "An Adaptive Large Neighborhood Search Heuristic for the Pickup and Delivery Problem with Time Windows" - ALNS for VRP

### Genetic Algorithms for VRP

- **Prins (2004)**: "A Simple and Effective Evolutionary Algorithm for the Vehicle Routing Problem" - Efficient GA
- **Nagata & Braysy (2009)**: "Edge Assembly Crossover for the Capacitated Vehicle Routing Problem" - State-of-the-art crossover

### Local Search Operators

- **Lin & Kernighan (1973)**: "An Effective Heuristic Algorithm for the Traveling-Salesman Problem" - k-opt origins
- **Or (1976)**: "Traveling Salesman-Type Combinatorial Problems and Their Relation to the Logistics of Regional Blood Banking" - Or-opt

### Open-Source VRP Solvers

| Solver | Language | Strengths |
|--------|----------|-----------|
| [Google OR-Tools](https://github.com/google/or-tools) | C++/Python | Production-ready, constraint programming |
| [VROOM](https://github.com/VROOM-Project/vroom) | C++ | Fast, practical, well-documented |
| [jsprit](https://github.com/graphhopper/jsprit) | Java | Flexible, ALNS-based |
| [OptaPlanner](https://github.com/kiegroup/optaplanner) | Java | Constraint satisfaction, enterprise features |
| [LKH-3](http://webhotel4.ruc.dk/~keld/research/LKH-3/) | C | State-of-the-art TSP/VRP solver |
| [HGS-CVRP](https://github.com/vidalt/HGS-CVRP) | C++ | State-of-the-art genetic algorithm for CVRP |

### Benchmarks

- **Solomon Instances**: Standard VRPTW benchmarks - [Link](https://www.sintef.no/projectweb/top/vrptw/solomon-benchmark/)
- **Gehring & Homberger**: Large-scale VRPTW instances
- **CVRPLIB**: Capacitated VRP benchmark library - [Link](http://vrp.atd-lab.inf.puc-rio.br/index.php/en/)
