//! Realistic routing tests using real Las Vegas locations and OSRM.
//!
//! These tests validate the full pipeline with real-world coordinates
//! and actual road network routing via OSRM.

mod fixtures;

use std::env;

use testcontainers::core::{IntoContainerPort, Mount};
use testcontainers::runners::SyncRunner;
use testcontainers::{Container, GenericImage, ImageExt, ReuseDirective, TestcontainersError};

use vrp_planner::osrm::{OsrmClient, OsrmConfig};
use vrp_planner::osrm_data::{GeofabrikRegion, OsrmDataset, OsrmDatasetConfig};
use vrp_planner::solver::{solve, SolveOptions};
use vrp_planner::traits::{AvailabilityProvider, Visit, VisitPinType, Visitor};

use fixtures::las_vegas_locations::{self, Location};

// ============================================================================
// Test Infrastructure
// ============================================================================

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct VisitId(String);

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct VisitorId(String);

struct RealVisit {
    id: VisitId,
    location: Location,
    duration_min: i32,
    pin_type: VisitPinType,
    pinned_visitor: Option<VisitorId>,
    committed_window: Option<(i32, i32)>,
    target_time: Option<i32>,
    required_capabilities: Vec<String>,
}

impl RealVisit {
    fn new(id: &str, location: Location) -> Self {
        Self {
            id: VisitId(id.to_string()),
            location,
            duration_min: 30,
            pin_type: VisitPinType::None,
            pinned_visitor: None,
            committed_window: None,
            target_time: None,
            required_capabilities: Vec::new(),
        }
    }

    fn duration(mut self, minutes: i32) -> Self {
        self.duration_min = minutes;
        self
    }

    fn pinned_to(mut self, visitor_id: &str) -> Self {
        self.pin_type = VisitPinType::Visitor;
        self.pinned_visitor = Some(VisitorId(visitor_id.to_string()));
        self
    }

    fn committed_window(mut self, start: i32, end: i32) -> Self {
        self.committed_window = Some((start, end));
        self
    }

    fn target_time(mut self, time: i32) -> Self {
        self.target_time = Some(time);
        self
    }
}

impl Visit for RealVisit {
    type Id = VisitId;
    type VisitorId = VisitorId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn scheduled_date(&self) -> Option<i64> {
        Some(1)
    }

    fn estimated_duration_minutes(&self) -> i32 {
        self.duration_min
    }

    fn committed_window(&self) -> Option<(i32, i32)> {
        self.committed_window
    }

    fn target_time(&self) -> Option<i32> {
        self.target_time
    }

    fn pin_type(&self) -> VisitPinType {
        self.pin_type
    }

    fn pinned_visitor(&self) -> Option<&Self::VisitorId> {
        self.pinned_visitor.as_ref()
    }

    fn pinned_date(&self) -> Option<i64> {
        None
    }

    fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    fn location(&self) -> (f64, f64) {
        self.location.coords()
    }
}

struct RealVisitor {
    id: VisitorId,
    start_location: Location,
    capabilities: Vec<String>,
}

impl RealVisitor {
    fn new(id: &str, start_location: Location) -> Self {
        Self {
            id: VisitorId(id.to_string()),
            start_location,
            capabilities: Vec::new(),
        }
    }
}

impl Visitor for RealVisitor {
    type Id = VisitorId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn start_location(&self) -> Option<(f64, f64)> {
        Some(self.start_location.coords())
    }

    fn end_location(&self) -> Option<(f64, f64)> {
        None
    }

    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

struct StandardAvailability;

impl AvailabilityProvider for StandardAvailability {
    type VisitorId = VisitorId;

    fn availability_for(&self, _visitor_id: &Self::VisitorId, _date: i64) -> Option<Vec<(i32, i32)>> {
        // 8am to 5pm
        Some(vec![(8 * 3600, 17 * 3600)])
    }
}

fn hours(h: i32) -> i32 {
    h * 3600
}

// ============================================================================
// OSRM Setup (reused from osrm_integration test)
// ============================================================================

fn osrm_container() -> Result<(Container<GenericImage>, OsrmClient), TestcontainersError> {
    let data_root = env::var("OSRM_DATA_DIR").unwrap_or_else(|_| "osrm-data".to_string());
    let region = GeofabrikRegion::new("north-america/us/nevada");
    let config = OsrmDatasetConfig::new(region, data_root);
    let dataset = OsrmDataset::ensure(&config)
        .map_err(|err| TestcontainersError::other(format!("OSRM prep failed: {:?}", err)))?;

    let mtime = std::fs::metadata(dataset.osrm_base.with_extension("osrm.partition"))
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let container_name = format!("osrm-nevada-mld-{}", mtime);

    let image = GenericImage::new("osrm/osrm-backend", "latest")
        .with_exposed_port(5000.tcp())
        .with_mount(Mount::bind_mount(
            dataset.data_dir.to_string_lossy().to_string(),
            "/data",
        ))
        .with_cmd(vec![
            "osrm-routed",
            "--algorithm",
            "mld",
            "/data/nevada-latest.osrm",
        ])
        .with_container_name(container_name)
        .with_startup_timeout(std::time::Duration::from_secs(30))
        .with_reuse(ReuseDirective::Always);

    let container = image.start()?;
    let port = container.get_host_port_ipv4(5000.tcp())?;
    let base_url = format!("http://127.0.0.1:{}", port);

    let osrm = OsrmClient::new(OsrmConfig {
        base_url,
        profile: "car".to_string(),
        timeout_secs: 30,
    }).map_err(|err| TestcontainersError::other(format!("OSRM client failed: {:?}", err)))?;

    Ok((container, osrm))
}

// ============================================================================
// Tests
// ============================================================================

/// Test with a small number of real locations to verify OSRM integration.
#[test]
fn test_small_route_with_osrm() {
    let (_container, osrm) = osrm_container().expect("start OSRM container");

    // Use geographically diverse locations
    let locations = las_vegas_locations::geographically_diverse_locations();

    // Create visits from first 6 locations
    let visits: Vec<RealVisit> = locations
        .iter()
        .take(6)
        .enumerate()
        .map(|(i, loc)| RealVisit::new(&format!("visit_{}", i), loc.clone()).duration(30))
        .collect();

    // Two technicians starting from different casinos
    let visitors = vec![
        RealVisitor::new("tech_1", las_vegas_locations::CASINOS[0].clone()), // Wynn
        RealVisitor::new("tech_2", las_vegas_locations::CASINOS[4].clone()), // Caesars
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &StandardAvailability,
        &osrm,
        SolveOptions::default(),
    );

    // All visits should be assigned
    assert!(
        result.unassigned.is_empty(),
        "All visits should be assigned, but {} were unassigned",
        result.unassigned.len()
    );

    // Both technicians should have some work
    let tech1_count = result
        .routes
        .iter()
        .find(|r| r.visitor_id.0 == "tech_1")
        .map(|r| r.visit_ids.len())
        .unwrap_or(0);
    let tech2_count = result
        .routes
        .iter()
        .find(|r| r.visitor_id.0 == "tech_2")
        .map(|r| r.visit_ids.len())
        .unwrap_or(0);

    println!("Tech 1 visits: {}", tech1_count);
    println!("Tech 2 visits: {}", tech2_count);

    // Verify total travel time is reasonable (not zero, not absurdly high)
    for route in &result.routes {
        if !route.visit_ids.is_empty() {
            println!(
                "{}: {} visits, {} seconds total travel",
                route.visitor_id.0,
                route.visit_ids.len(),
                route.total_travel_time
            );
            assert!(
                route.total_travel_time > 0,
                "Travel time should be positive"
            );
            // Las Vegas is ~30km across, so max reasonable travel should be under 2 hours
            assert!(
                route.total_travel_time < 7200,
                "Travel time seems too high: {} seconds",
                route.total_travel_time
            );
        }
    }
}

/// Test with 20 visits across 3 technicians - a realistic day.
#[test]
fn test_medium_route_with_osrm() {
    let (_container, osrm) = osrm_container().expect("start OSRM container");

    // Get 20 locations from different areas
    let all_locs = las_vegas_locations::all_locations();
    let visits: Vec<RealVisit> = all_locs
        .iter()
        .take(20)
        .enumerate()
        .map(|(i, loc)| {
            RealVisit::new(&format!("visit_{}", i), loc.clone())
                .duration(20 + (i as i32 % 3) * 10) // 20-40 minute visits
        })
        .collect();

    // Three technicians starting from different locations
    let visitors = vec![
        RealVisitor::new("north_tech", las_vegas_locations::NORTH_VEGAS_LOCATIONS[0].clone()),
        RealVisitor::new("strip_tech", las_vegas_locations::CASINOS[0].clone()),
        RealVisitor::new("henderson_tech", las_vegas_locations::HENDERSON_LOCATIONS[0].clone()),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &StandardAvailability,
        &osrm,
        SolveOptions::default(),
    );

    // Count assigned vs unassigned
    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();
    println!(
        "Assigned: {}, Unassigned: {}",
        total_assigned,
        result.unassigned.len()
    );

    // Most visits should be assigned (might have some infeasible due to time constraints)
    assert!(
        total_assigned >= 15,
        "At least 15 of 20 visits should be assigned, got {}",
        total_assigned
    );

    // Print route summary
    for route in &result.routes {
        println!(
            "{}: {} visits, {:.1} min travel",
            route.visitor_id.0,
            route.visit_ids.len(),
            route.total_travel_time as f64 / 60.0
        );
    }
}

/// Test with committed time windows.
#[test]
fn test_time_windows_with_osrm() {
    let (_container, osrm) = osrm_container().expect("start OSRM container");

    let locations = las_vegas_locations::STRIP_RESTAURANTS;

    // Create visits with specific time windows
    let visits = vec![
        RealVisit::new("morning", locations[0].clone())
            .duration(30)
            .committed_window(hours(8), hours(10)), // 8-10am
        RealVisit::new("midday", locations[1].clone())
            .duration(30)
            .committed_window(hours(11), hours(13)), // 11am-1pm
        RealVisit::new("afternoon", locations[2].clone())
            .duration(30)
            .committed_window(hours(14), hours(16)), // 2-4pm
        RealVisit::new("flexible", locations[3].clone()).duration(30), // no window
    ];

    let visitors = vec![RealVisitor::new(
        "tech",
        las_vegas_locations::CASINOS[0].clone(),
    )];

    let result = solve(
        1,
        &visits,
        &visitors,
        &StandardAvailability,
        &osrm,
        SolveOptions::default(),
    );

    // All should be assigned
    assert!(
        result.unassigned.is_empty(),
        "All visits should be assigned"
    );

    // Verify estimated windows respect committed windows
    let route = &result.routes[0];
    for (i, visit) in visits.iter().enumerate() {
        if let Some((commit_start, commit_end)) = visit.committed_window {
            let (est_start, _est_end) = route.estimated_windows[route
                .visit_ids
                .iter()
                .position(|id| id.0 == visit.id.0)
                .unwrap()];
            assert!(
                est_start >= commit_start && est_start <= commit_end,
                "Visit {} estimated start {} should be within committed window [{}, {}]",
                visit.id.0,
                est_start,
                commit_start,
                commit_end
            );
        }
    }
}
