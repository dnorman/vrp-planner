# VRP Solver Comprehensive Test Plan

This document outlines all scenarios a service company might encounter that the routing solver must handle correctly.

## Test Coverage Summary

### vrp-planner (Solver)

| Category | Tests | Status |
|----------|-------|--------|
| Basic Assignment | 6 | ✓ |
| Time Windows | 10 | ✓ |
| Pinning | 7 | ✓ |
| Capabilities | 5 | ✓ |
| Sick Day / Unavailable | 7 | ✓ |
| Running Late / Delayed | 4 | ✓ |
| Variable Availability | 3 | ✓ |
| Stability | 2 | ✓ |
| Geographic | 3 | ✓ |
| Workload Balance | 1 | ✓ |
| Local Search | 2 | ✓ |
| Quality Benchmarks | 3 | ✓ |
| Scale / Stress | 4 | ✓ |
| Composite Scenarios | 2 | ✓ |
| OSRM Integration | 4 | ✓ |
| Smoke Tests | 1 | ✓ |
| Haversine Unit Tests | 5 | ✓ |
| Fixture Validation | 2 | ✓ |

**vrp-planner Total: 70 tests**

### properlydone-platform-routing (E2E)

| Category | Tests | Status |
|----------|-------|--------|
| Platform E2E | 9 | ✓ |
| Recalculation Tests | 7 | ✓ |
| Watcher Integration | 5 | Planned |
| Day-of-Service E2E | 5 | Planned |

**Platform Total: 16 passing, 10 planned**

---

**Combined: 86 tests (70 solver + 16 platform)**

---

## 1. Basic Assignment Scenarios

- [x] `test_single_visit_single_visitor` - One visit assigned to one available tech
- [x] `test_multiple_visits_sequenced` - Multiple visits assigned in sequence
- [x] `test_visits_distributed_across_visitors` - Visits spread across multiple techs
- [x] `test_empty_visits` - No visits to assign (should return empty routes)
- [x] `test_no_visitors` - No techs available (all visits unassigned)

---

## 2. Time Window Scenarios

### Committed Windows (Hard Constraints)
- [x] `test_committed_window_respected` - Visit scheduled within window
- [x] `test_committed_window_infeasible` - Window impossible → unassigned
- [x] `test_narrow_committed_window_30_minutes` - 30-minute window (tight constraint)
- [x] `test_visit_at_day_start` - Committed window at 8:00am
- [x] `test_visit_at_day_end` - Committed window ends at 5:00pm
- [x] `test_visit_exactly_fills_window` - Perfect fit, no slack

### Target Times (Soft Constraints)
- [x] `test_target_time_affects_cost` - Target time influences sequencing
- [x] `test_target_time_sequencing_with_local_search` - Local search optimizes toward targets

### Service Duration
- [x] `test_short_visit_15_minutes` - Quick check/inspection
- [x] `test_long_visit_3_hours` - Major repair or installation
- [x] `test_mixed_durations_same_route` - Mix of short and long visits
- [x] `test_visit_duration_exceeds_remaining_window` - Visit too long for day

---

## 3. Pinning Scenarios

### Visitor Pinning
- [x] `test_pinned_to_visitor` - Visit must go to specific tech
- [x] `test_pinned_visitor_missing` - Pinned tech doesn't exist
- [x] `test_visitor_unavailable` - Pinned tech unavailable
- [x] `test_multiple_visits_pinned_same_tech` - Several customers request same tech

### Date Pinning
- [x] `test_pinned_to_date_matching` - Visit for specific date only
- [x] `test_pinned_visitor_and_wrong_date` - Visit pinned to different date → excluded

### Combined Pinning
- [x] `test_pinned_to_visitor_and_date` - Must be specific tech on specific day

---

## 4. Capability / Skill Matching

- [x] `test_capability_superset_match` - Tech has all required skills
- [x] `test_no_capable_visitor` - No tech has required skill
- [x] `test_multiple_techs_same_capability_choose_closest` - Choose closest
- [x] `test_visit_requires_multiple_capabilities` - Plumbing AND electrical
- [x] `test_rare_skill_only_one_tech` - Only one tech can do certain visits

---

## 5. Sick Day / Unavailable Scenarios

- [x] `test_reassignment_when_visitor_calls_in_sick` - All visits reassigned
- [x] `test_partial_reassignment_multiple_visitors_sick` - One sick, one healthy
- [x] `test_reassignment_respects_capabilities` - Reassigned to capable backup only
- [x] `test_reassignment_when_no_capable_backup` - No backup → unassigned
- [x] `test_two_of_three_techs_sick` - Heavy load on remaining tech
- [x] `test_all_techs_unavailable` - All visits unassigned with reason

---

## 6. Running Late / Delayed Start

- [x] `test_running_late_visits_rescheduled` - Visits pushed to later times
- [x] `test_running_late_some_visits_reassigned` - Committed window visits reassigned
- [x] `test_running_late_cascading_reassignment` - Workload cascades to others
- [x] `test_running_late_no_one_can_cover` - No backup for early window

---

## 7. Variable Availability / Part-Time

- [x] `test_part_time_morning_only` - Tech works 8am-12pm
- [x] `test_staggered_start_times` - Team starts at different times
- [x] `test_mid_day_break` - Placeholder for break handling

---

## 8. Stability / Reassignment Penalty

- [x] `test_stability_penalty_prefers_current_assignment` - Visits stay with current tech

---

## 9. Geographic / Travel Time

- [x] `test_geographic_clustering` - Visits in same area grouped together
- [x] `test_minimize_backtracking` - Don't zigzag across service area
- [x] `test_multiple_visits_same_address` - Two services at one property

---

## 10. Workload Balance

- [x] `test_workload_roughly_balanced` - Similar number of visits per tech

---

## 11. Local Search

- [x] `test_two_opt_improves_crossing_routes` - 2-opt removes crossings
- [x] `test_relocate_balances_routes` - Relocate moves visits between routes

---

## 12. Quality Benchmarks

- [x] `test_local_search_improves_solution_quality` - 9.3% travel time reduction
- [x] `test_travel_efficiency_geographic_clusters` - 100% correct clustering
- [x] `test_solution_determinism` - Consistent results across runs

---

## 13. Scale / Stress Tests

- [x] `test_50_visits_5_visitors` - 50 visits in ~50ms
- [x] `test_100_visits_10_visitors` - 100 visits in ~230ms
- [x] `test_140_visits_14_visitors` - 140 visits in ~444ms
- [x] `test_200_visits_20_visitors_stress` - 200 visits in ~928ms

---

## 14. Composite Real-World Scenarios

- [x] `test_realistic_service_day` - 40 visits with VIPs, repairs, quotes, capabilities
- [x] `test_worst_case_all_constraints` - Stress test with all constraint types

---

## Future Work (v2)

### Priority / Urgency
- [ ] `test_urgent_visit_scheduled_first` - Emergency gets priority
- [ ] `test_urgent_bumps_flexible` - Flexible visits rescheduled for emergency

### VIP Customers
- [ ] `test_vip_preferred_time` - VIP gets requested time slot
- [ ] `test_vip_never_unassigned` - VIP visits assigned even if suboptimal

### Break Handling - COMPLETE ✓
- [x] `test_route_optimization_with_break` - Lunch breaks handled correctly (in platform E2E tests)
- Multiple availability windows supported via `Vec<TimeWindow>`

### Working Hours Limits
- [ ] `test_max_hours_per_day` - Tech has hour limit (e.g., 8 hours)
- [ ] `test_overtime_avoidance` - Prefer not to exceed standard hours

---

## 15. OSRM Integration Tests

Tests that validate the full pipeline with real-world coordinates and actual road network routing via OSRM.

- [x] `osrm_table_returns_matrix` - OSRM table endpoint returns valid distance matrix
- [x] `test_small_route_with_osrm` - 6 visits with real Las Vegas coordinates via OSRM
- [x] `test_medium_route_with_osrm` - 20 visits across 3 technicians with OSRM routing
- [x] `test_time_windows_with_osrm` - Committed windows validated with real road travel times

---

## 16. Smoke Tests

- [x] `honors_pinned_visitor` - Basic smoke test for pinned visitor functionality

---

## 17. Haversine Unit Tests

Unit tests for the Haversine fallback distance matrix provider.

- [x] `test_haversine_same_point` - Same point returns ~0 distance
- [x] `test_haversine_known_distance` - Las Vegas to LA returns ~370km
- [x] `test_matrix_diagonal_is_zero` - Diagonal of matrix is zero
- [x] `test_matrix_symmetric` - Matrix is symmetric
- [x] `test_reasonable_travel_time` - 10km at 40km/h = 900 seconds

---

## 18. Fixture Validation Tests

Tests that validate test fixtures are correctly set up.

- [x] `test_all_locations_count` - Las Vegas fixture has expected location count
- [x] `test_coordinates_in_vegas_area` - All coordinates are within Las Vegas bounds

---

---

## 19. Platform E2E Tests (properlydone-platform-routing)

End-to-end tests that exercise the full integration with ankurah database, adapters, and result persistence.

Located in `properlydone-platform-routing/server/tests/`.

### Existing Tests

- [x] `test_route_optimization_basic` - Basic optimization with ankurah context
- [x] `test_route_optimization_with_capabilities` - Capability matching persisted correctly
- [x] `test_route_optimization_with_committed_windows` - Time windows respected
- [x] `test_route_optimization_with_pinning` - Pinned visits assigned correctly
- [x] `test_route_optimization_unassigned_reasons` - Unassigned reasons persisted
- [x] `test_route_optimization_creates_route_plan` - RoutePlan records created
- [x] `test_visit_status_transitions_for_recalculation` - Status change triggers recalc
- [x] `test_route_optimization_with_availability` - Availability windows respected
- [x] `test_route_optimization_with_break` - Lunch breaks handled correctly

**Total: 9 tests passing**

### Service Crate Tests - COMPLETE ✓

Tests for `services/route-planner` crate.

#### Recalculation Tests

- [x] `test_recalc_tech_arrives_early` - ETAs shift earlier for downstream visits
- [x] `test_recalc_tech_arrives_late_with_violation` - ETAs shift later, violations flagged
- [x] `test_recalc_visit_completes_faster` - Time gained propagates
- [x] `test_recalc_visit_skipped` - Remaining visits recalculated
- [x] `test_recalc_no_change_when_on_schedule` - Minimal delay when on time
- [x] `test_apply_recalculated_etas` - Updated windows persisted to database
- [x] `test_recalc_multiple_completed` - Handles several done visits

#### Watcher Integration Tests

- [ ] `test_watcher_triggers_on_status_change_to_inprogress` - Recalc when tech arrives
- [ ] `test_watcher_triggers_on_status_change_to_completed` - Recalc when visit done
- [ ] `test_watcher_triggers_on_status_change_to_skipped` - Recalc when skipped
- [ ] `test_watcher_ignores_scheduled_visits` - No recalc for scheduled visits
- [ ] `test_watcher_ignores_visits_without_route_plan` - No recalc for unassigned

#### End-to-End Day-of-Service Scenarios

- [ ] `test_e2e_tech_running_late_all_day` - ETAs cascade for entire route
- [ ] `test_e2e_tech_catches_up_after_short_visit` - Delay recovery tracked
- [ ] `test_e2e_multiple_techs_running_concurrently` - Parallel recalculations
- [ ] `test_e2e_mixed_status_transitions` - Complex sequence of status changes
- [ ] `test_e2e_commitment_notification_created` - Dispatcher notified of violations

---

## Bug Fixes Applied

- **Relocate operator now respects pinned visits** - Local search was moving pinned visits between routes, violating customer technician requests. Fixed by checking pin_type before inter-route moves.
