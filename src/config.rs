// ============================================================================
// SecCamCloud - Configuration Management Module
// Version: 1.0.0
// Author: Michael Lauzon
// Rust Edition: 2024
// License: GPLv2
// ============================================================================

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use log::{info, warn};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Click point with coordinates and descriptive name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClickPoint {
    pub name: String,
    pub x: i32,
    pub y: i32,
}

impl ClickPoint {
    pub fn new(name: impl Into<String>, x: i32, y: i32) -> Self {
        Self {
            name: name.into(),
            x,
            y,
        }
    }
}

/// Default automation click points
lazy_static! {
    pub static ref DEFAULT_POINTS: Vec<ClickPoint> = vec![
        ClickPoint::new("Step 1", 3514, 1640),
        ClickPoint::new("Step 2 (date field)", 1775, 596),
        ClickPoint::new("Step 3", 1474, 1649),
        ClickPoint::new("Step 5", 2875, 1640),
        ClickPoint::new("Step 7", 2674, 1640),
        ClickPoint::new("Step 8", 2066, 1100),
    ];
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub points: Vec<ClickPoint>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            points: DEFAULT_POINTS.clone(),
        }
    }
}

// ============================================================================
// CONFIGURATION MANAGEMENT
// ============================================================================

/// Load click points from configuration
pub fn load_points() -> Vec<ClickPoint> {
    // Try JSON file first (preferred format)
    let json_path = Path::new("clickpoints.json");
    if json_path.exists() {
        if let Ok(file) = File::open(json_path) {
            if let Ok(points) = serde_json::from_reader::<_, Vec<ClickPoint>>(BufReader::new(file)) {
                info!("Loaded {} click points from clickpoints.json", points.len());
                return points;
            }
        }
        warn!("Failed to parse clickpoints.json");
    }
    
    // Fallback to confy configuration
    match confy::load::<AppConfig>("SecCamCloud", None) {
        Ok(cfg) => {
            info!("Loaded {} click points from confy config", cfg.points.len());
            cfg.points
        }
        Err(e) => {
            warn!("Failed to load confy config: {}", e);
            info!("Using default click points");
            DEFAULT_POINTS.clone()
        }
    }
}

/// Save click points to configuration
pub fn save_points(points: &[ClickPoint]) {
    // Save to JSON file (preferred format)
    let json_path = Path::new("clickpoints.json");
    if let Ok(file) = File::create(json_path) {
        if serde_json::to_writer_pretty(file, points).is_ok() {
            info!("Saved {} click points to clickpoints.json", points.len());
        } else {
            warn!("Failed to serialize points to JSON");
        }
    } else {
        warn!("Failed to create clickpoints.json");
    }
    
    // Also save to confy as backup
    let cfg = AppConfig {
        points: points.to_vec(),
    };
    
    if let Err(e) = confy::store("SecCamCloud", None, cfg) {
        warn!("Failed to save confy config: {}", e);
    } else {
        info!("Saved backup config to confy");
    }
}
