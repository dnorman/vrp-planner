# Problem Statement

## Overview

Build a vehicle routing planner for field service businesses.

## The Problem

**Vehicle Routing Problem (VRP)**: Given a set of visits that need to happen and a set of drivers, find optimal routes for each driver that:
- Minimize total travel time/distance
- Respect all constraints
- Maximize completed visits

## Our Constraints

| Constraint | Description | Priority |
|------------|-------------|----------|
| Multiple drivers | Fleet of technicians, each with their own vehicle | Must have |
| Pinned to driver | Some visits MUST be done by a specific driver (customer preference, expertise) | Must have |
| Pinned to time | Some visits have fixed appointment times | Must have |
| Time windows | Visits may have "arrive between X and Y" constraints | Should have |
| Driver shifts | Drivers have start/end times for their workday | Should have |
| Driver breaks | Lunch breaks, rest periods | Nice to have |
| Driver start locations | Drivers may start from home, not a central depot | Nice to have |
| Capacities | Equipment/chemical limits per vehicle | Nice to have |

## Non-Goals (for now)

- Real-time re-routing during the day
- Multi-day planning
- Dynamic customer requests
- Integration with mapping services (we'll use Euclidean distance initially)

## Success Criteria

1. Solver produces valid routes (all constraints respected)
2. Routes are reasonably efficient (not wildly suboptimal)
3. Pinned visits are honored absolutely
4. Solver runs fast enough for interactive use (<30 seconds for typical problems)

## Typical Problem Size

- 3-10 drivers
- 20-100 visits per day
- 5-20% of visits pinned to specific drivers
- 5-10% of visits pinned to specific times
