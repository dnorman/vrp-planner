use std::env;

use testcontainers::core::{IntoContainerPort, Mount};
use testcontainers::ReuseDirective;
use testcontainers::runners::SyncRunner;
use testcontainers::{Container, GenericImage, ImageExt, TestcontainersError};

use vrp_planner::osrm_data::{GeofabrikRegion, OsrmDataset, OsrmDatasetConfig};
use vrp_planner::osrm::{OsrmClient, OsrmConfig};
use vrp_planner::traits::DistanceMatrixProvider;

fn osrm_container() -> Result<(Container<GenericImage>, String), TestcontainersError> {
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

    Ok((container, base_url))
}

#[test]
fn osrm_table_returns_matrix() {
    let (container, base_url) = osrm_container().expect("start OSRM container");

    let profile = "car".to_string();
    let config = OsrmConfig {
        base_url: base_url.clone(),
        profile: profile.clone(),
        timeout_secs: 10,
    };
    let client = OsrmClient::new(config).expect("build OSRM client");

    let locations = vec![
        (36.1147, -115.1728),
        (36.1727, -115.1580),
        (36.1215, -115.1739),
    ];

    let matrix = {
        let start = std::time::Instant::now();
        let mut last = Vec::new();
        while start.elapsed() < std::time::Duration::from_secs(15) {
            last = client.matrix_for(&locations);
            if last.len() == locations.len() && !last.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        last
    };
    if matrix.is_empty() {
        let coords = locations
            .iter()
            .map(|(lat, lng)| format!("{:.6},{:.6}", lng, lat))
            .collect::<Vec<_>>()
            .join(";");
        let url = format!(
            "{}/table/v1/{}/{}?annotations=duration",
            base_url, profile, coords
        );
        match reqwest::blocking::get(&url) {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().unwrap_or_else(|_| "<no body>".to_string());
                eprintln!("OSRM status: {}", status);
                eprintln!("OSRM body: {}", body);
            }
            Err(err) => {
                eprintln!("OSRM request error: {}", err);
            }
        }
        if let Ok(stdout) = container.stdout_to_vec() {
            if !stdout.is_empty() {
                eprintln!("OSRM stdout:\n{}", String::from_utf8_lossy(&stdout));
            }
        }
        if let Ok(stderr) = container.stderr_to_vec() {
            if !stderr.is_empty() {
                eprintln!("OSRM stderr:\n{}", String::from_utf8_lossy(&stderr));
            }
        }
    }
    assert_eq!(matrix.len(), locations.len());
    assert_eq!(matrix[0].len(), locations.len());

    drop(container);
}
