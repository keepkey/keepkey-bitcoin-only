use semver::{Version};

// Compares two version strings. Returns Ok(true) if current_version < required_version.
// Returns Err if parsing fails.
pub fn is_version_older(current_version_str: &str, required_version_str: &str) -> Result<bool, String> {
    let current = Version::parse(current_version_str)
        .map_err(|e| format!("Failed to parse current version '{}': {}", current_version_str, e))?;
    let required = Version::parse(required_version_str)
        .map_err(|e| format!("Failed to parse required version '{}': {}", required_version_str, e))?;
    Ok(current < required)
}

// You can add other utility functions here as needed.
