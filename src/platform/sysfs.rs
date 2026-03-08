use std::fs;
use std::path::{Path, PathBuf};

pub fn read_string_optional(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty()
            || trimmed == "N/A"
            || trimmed == "To Be Filled By O.E.M."
            || trimmed == "Default string"
            || trimmed == "Not Specified"
        {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub fn read_u64_optional(path: &Path) -> Option<u64> {
    read_string_optional(path).and_then(|s| parse_int_flexible(&s).ok())
}

pub fn read_u32_optional(path: &Path) -> Option<u32> {
    read_u64_optional(path).map(|v| v as u32)
}

pub fn read_link_basename(path: &Path) -> Option<String> {
    fs::read_link(path)
        .ok()
        .and_then(|target| target.file_name().map(|n| n.to_string_lossy().to_string()))
}

pub fn glob_paths(pattern: &str) -> Vec<PathBuf> {
    glob::glob(pattern)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .collect()
}

fn parse_int_flexible(s: &str) -> Result<u64, std::num::ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16)
    } else {
        s.parse::<u64>()
    }
}
