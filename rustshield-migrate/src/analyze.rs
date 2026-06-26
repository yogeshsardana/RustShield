// RustShield — analyze: C driver source code analyzer

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Result of analyzing a C kernel driver.
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub driver_name: String,
    pub source_path: String,
    pub total_lines: u64,
    pub io_operations: u64,
    pub state_variables: u64,
    pub state_bytes: u64,
    pub lock_acquisition_points: u64,
    pub dma_regions: Vec<DmaRegionInfo>,
    pub function_count: u64,
    pub ndoc_operations: Vec<String>,
    pub migration_score: u32,
    pub verus_hints: Vec<VerusHint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmaRegionInfo {
    pub name: String,
    pub size_bytes: u64,
    pub direction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerusHint {
    pub function: String,
    pub invariant_type: String,
    pub annotation: String,
}

/// Analyze a C driver source tree and produce a migration assessment.
pub fn analyze_c_driver(path: &str) -> anyhow::Result<AnalysisResult> {
    let driver_path = Path::new(path);
    if !driver_path.exists() {
        anyhow::bail!("Driver path does not exist: {}", path);
    }

    // In production, this uses tree-sitter or similar to parse C source.
    // For the prototype, we return a representative analysis.
    let driver_name = driver_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(AnalysisResult {
        driver_name,
        source_path: path.to_string(),
        total_lines: count_c_lines(driver_path),
        io_operations: 0,
        state_variables: 0,
        state_bytes: 0,
        lock_acquisition_points: 0,
        dma_regions: Vec::new(),
        function_count: 0,
        ndoc_operations: Vec::new(),
        migration_score: 0,
        verus_hints: Vec::new(),
    })
}

fn count_c_lines(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("c") {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    total += content.lines().count() as u64;
                }
            } else if p.is_dir() {
                total += count_c_lines(&p);
            }
        }
    }
    total
}

pub fn write_report(result: &AnalysisResult, path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn read_report(path: &str) -> anyhow::Result<AnalysisResult> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}
