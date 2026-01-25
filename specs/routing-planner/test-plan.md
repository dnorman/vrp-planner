# VRP Solver Comprehensive Test Plan

This document outlines all scenarios a service company might encounter that the routing solver must handle correctly.

## Test Coverage Summary

| Category | Scenarios | Implemented |
|----------|-----------|-------------|
| Basic Assignment | 6 | ✓ |
| Time Windows | 10 | ✓ |
| Pinning | 6 | ✓ |
| Capabilities | 5 | ✓ |
| Sick Day / Unavailable | 7 | ✓ |
| Running Late / Delayed | 4 | ✓ |
| Variable Availability | 3 | ✓ |
| Stability | 2 | ✓ |
| Geographic | 3 | ✓ |
| Workload Balance | 1 | ✓ |
| Local Search | 2 | ✓ |
| Edge Cases | 3 | ✓ |
| Scale | 2 | ✓ |

**Total: 51 tests passing**

Legend: ✓ = Implemented, ○ = Not yet implemented

---

## 1. Basic Assignment Scenarios

### 1.1 Simple Assignment
- [x] `test_single_visit_single_visitor` - One visit assigned to one available tech
- [x] `test_multiple_visits_sequenced` - Multiple visits assigned in sequence
- [x] `test_visits_distributed_across_visitors` - Visits spread across multiple techs
- [x] `test_empty_visits` - No visits to assign (should return empty routes)
- [x] `test_no_visitors` - No techs available (all visits unassigned)

### 1.2 Visit Types (Note: visit generation is external, solver just handles routing)
- [ ] `test_quote_appointment` - Short duration, flexible window
- [ ] `test_repair_visit` - Longer duration, may have committed window
- [ ] `test_recurring_weekly_batch` - Multiple recurring visits on same day

---

## 2. Time Window Scenarios

### 2.1 Committed Windows (Hard Constraints)
- [x] `test_committed_window_respected` - Visit scheduled within window
- [x] `test_committed_window_infeasible` - Window impossible → unassigned
- [x] `test_narrow_committed_window_30_minutes` - 30-minute window (tight constraint)
- [x] `test_visit_at_day_start` - Committed window at 8:00am
- [x] `test_visit_at_day_end` - Committed window ends at 5:00pm
- [x] `test_visit_exactly_fills_window` - Perfect fit, no slack

### 2.2 Target Times (Soft Constraints)
- [x] `test_target_time_affects_cost` - Target time influences sequencing
- [x] `test_target_time_sequencing_with_local_search` - Local search optimizes toward targets
- [ ] `test_early_morning_target` - Customer prefers 8am visit
- [ ] `test_late_afternoon_target` - Customer prefers 4pm visit
- [ ] `test_conflicting_targets` - Multiple visits with conflicting target times

### 2.3 Service Duration
- [x] `test_short_visit_15_minutes` - Quick check/inspection
- [x] `test_long_visit_3_hours` - Major repair or installation
- [x] `test_mixed_durations_same_route` - Mix of short and long visits
- [x] `test_visit_duration_exceeds_remaining_window` - Visit too long for day

---

## 3. Pinning Scenarios

### 3.1 Visitor Pinning (Customer Requests Specific Tech)
- [x] `test_pinned_to_visitor` - Visit must go to specific tech
- [x] `test_pinned_visitor_missing` - Pinned tech doesn't exist
- [x] `test_visitor_unavailable` - Pinned tech unavailable
- [x] `test_multiple_visits_pinned_same_tech` - Several customers request same tech

### 3.2 Date Pinning
- [x] `test_pinned_to_date_matching` - Visit for specific date only
- [x] `test_pinned_visitor_and_wrong_date` - Visit pinned to different date → excluded

### 3.3 Combined Pinning
- [x] `test_pinned_to_visitor_and_date` - Must be specific tech on specific day

### 3.4 Time Pinning (Future)
- [ ] `test_pinned_to_exact_time` - Customer insists on 10am exactly
- [ ] `test_pinned_time_conflicts_with_another` - Two visits pinned to same time

---

## 4. Capability / Skill Matching Scenarios

### 4.1 Single Capability
- [x] `test_capability_superset_match` - Tech has all required skills
- [x] `test_no_capable_visitor` - No tech has required skill
- [x] `test_multiple_techs_same_capability_choose_closest` - Choose closest

### 4.2 Multiple Capabilities
- [x] `test_visit_requires_multiple_capabilities` - Plumbing AND electrical
- [x] `test_rare_skill_only_one_tech` - Only one tech can do certain visits

### 4.3 Capability-Based Routing
- [ ] `test_capability_clustering` - Group similar visits to specialists
- [ ] `test_specialist_vs_generalist` - Prefer specialist when available

---

## 5. Sick Day / Unavailable Scenarios

### 5.1 Full Day Unavailable
- [x] `test_reassignment_when_visitor_calls_in_sick` - All visits reassigned
- [x] `test_partial_reassignment_multiple_visitors_sick` - One sick, one healthy
- [x] `test_reassignment_respects_capabilities` - Reassigned to capable backup only
- [x] `test_reassignment_when_no_capable_backup` - No backup → unassigned

### 5.2 Multiple Techs Unavailable
- [x] `test_two_of_three_techs_sick` - Heavy load on remaining tech
- [x] `test_all_techs_unavailable` - All visits unassigned with reason
- [ ] `test_only_capable_tech_sick` - Specialized visits unassigned (covered by reassignment_when_no_capable_backup)

---

## 6. Running Late / Delayed Start Scenarios

### 6.1 Delayed Start
- [x] `test_running_late_visits_rescheduled` - Visits pushed to later times
- [x] `test_running_late_some_visits_reassigned` - Committed window visits reassigned
- [x] `test_running_late_cascading_reassignment` - Workload cascades to others
- [x] `test_running_late_no_one_can_cover` - No backup for early window

### 6.2 Early Departure (Future)
- [ ] `test_early_departure_visits_reassigned` - Tech leaves at 2pm, afternoon visits move
- [ ] `test_early_departure_last_visit_squeezed` - Visit fits if squeezed before departure

---

## 7. Variable Availability / Part-Time Scenarios

- [x] `test_part_time_morning_only` - Tech works 8am-12pm
- [x] `test_staggered_start_times` - Team starts at different times
- [x] `test_mid_day_break` - Placeholder for break handling (future)
- [ ] `test_part_time_afternoon_only` - Tech works 1pm-5pm
- [ ] `test_different_end_times` - Some techs work longer days

---

## 8. Stability / Reassignment Penalty Scenarios

### 8.1 Minimizing Churn
- [x] `test_stability_penalty_prefers_current_assignment` - Visits stay with current tech
- [ ] `test_stability_with_minor_improvement` - Don't reassign for small gains
- [ ] `test_stability_yields_to_major_improvement` - Reassign for significant gains

### 8.2 Customer Relationship Continuity (Future)
- [ ] `test_recurring_customer_same_tech` - Same tech week after week
- [ ] `test_new_customer_any_tech` - No preference for new customers
- [ ] `test_customer_requested_change` - Override stability when customer asks

---

## 9. Geographic / Travel Time Scenarios

### 9.1 Clustering
- [x] `test_geographic_clustering` - Visits in same area grouped together
- [x] `test_minimize_backtracking` - Don't zigzag across service area

### 9.2 Same Location
- [x] `test_multiple_visits_same_address` - Two services at one property

### 9.3 Multi-Area Routing (Future)
- [ ] `test_visits_in_different_cities` - Henderson vs Las Vegas
- [ ] `test_tech_assigned_to_region` - Techs have home territories
- [ ] `test_cross_region_overflow` - Overflow to adjacent region's tech

---

## 10. Workload Balance Scenarios

### 10.1 Fair Distribution
- [x] `test_workload_roughly_balanced` - Similar number of visits per tech
- [ ] `test_even_duration_distribution` - Similar total hours per tech
- [ ] `test_unbalanced_when_necessary` - Accept imbalance for constraints

### 10.2 Capacity Limits (Future)
- [ ] `test_max_visits_per_day` - Tech has visit limit (e.g., 12 max)
- [ ] `test_max_hours_per_day` - Tech has hour limit (e.g., 8 hours)
- [ ] `test_overtime_avoidance` - Prefer not to exceed standard hours

---

## 11. Local Search Tests

- [x] `test_two_opt_improves_crossing_routes` - 2-opt removes crossings
- [x] `test_relocate_balances_routes` - Relocate moves visits between routes

---

## 12. Scale Tests

- [x] `test_50_visits_5_visitors` - Medium scale
- [x] `test_100_visits_10_visitors` - Full scale with timing assertion (<30s, achieves ~230ms)
- [ ] `test_200_visits_20_visitors` - Stress test (larger company)

---

## 13. Priority / Urgency Scenarios (Future)

### 13.1 Emergency Visits
- [ ] `test_urgent_visit_scheduled_first` - Emergency gets priority
- [ ] `test_urgent_bumps_flexible` - Flexible visits rescheduled for emergency
- [ ] `test_multiple_emergencies` - Multiple urgents on same day

### 13.2 VIP Customers
- [ ] `test_vip_preferred_time` - VIP gets requested time slot
- [ ] `test_vip_assigned_senior_tech` - Best tech for important customers
- [ ] `test_vip_never_unassigned` - VIP visits assigned even if suboptimal

---

## 14. Real-World Composite Scenarios (Future)

### 14.1 Typical Monday Morning
```
Scenario: 5 techs, 40 visits
- 35 recurring weekly services
- 3 repair callbacks from Friday
- 2 new customer quotes
- 1 tech running 30 min late
- 2 visits have committed morning windows
```
- [ ] `test_typical_monday_scenario`

### 14.2 Emergency Day
```
Scenario: 3 emergencies arrive throughout day
- Normal schedule of 30 visits across 4 techs
- Emergency 1: Pool pump failure at 9am
- Emergency 2: Heater issue at 11am
- Emergency 3: Leak at 2pm
- Must fit emergencies without missing committed windows
```
- [ ] `test_emergency_day_scenario`

### 14.3 Short-Staffed Day
```
Scenario: 2 of 5 techs call in sick
- 45 visits scheduled
- Must identify which visits get unassigned
- Prioritize committed windows and VIP customers
```
- [ ] `test_short_staffed_scenario`

---

## Implementation Priority

### Phase 1: Core Functionality ✓ COMPLETE
- [x] Basic assignment
- [x] Sick day handling
- [x] Running late handling
- [x] Capability matching
- [x] Time windows (committed)
- [x] Stability penalty
- [x] Scale tests
- [x] Pinning (visitor, date, combined)
- [x] Variable availability
- [x] Geographic clustering
- [x] Workload balance

### Phase 2: Operational Robustness
- [ ] Target time optimization (partially done)
- [ ] Priority/urgency handling
- [ ] VIP customer handling
- [ ] Break handling within day

### Phase 3: Advanced Features
- [ ] Composite real-world scenarios
- [ ] Multi-region routing
- [ ] Capacity limits

---

## Test Data Requirements

### Fixtures Available
- [x] Las Vegas area coordinates (100+ POIs)
- [x] ManhattanMatrix for predictable testing
- [x] TestAvailability with per-visitor overrides
- [x] TestVisit with capabilities, windows, pinning
- [x] TestVisitor with capabilities, start locations

### Mock Data Generators (Future)
- [ ] Random visit generator with realistic distribution
- [ ] Random availability generator (sick days, late starts)
- [ ] Realistic capability distribution across techs
