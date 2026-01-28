use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub home_assistant: HomeAssistantConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HomeAssistantConfig {
    pub url: String,
    pub token: String,
    pub entities: Entities,
    #[allow(dead_code)]
    pub max_solar_power: Option<f64>,
    pub battery_float_voltage: Option<f64>,
    pub battery_capacity_kwh: Option<f64>,
    #[serde(default)]
    pub history_duration: Option<String>,
    #[serde(default)]
    pub history_seconds: Option<usize>, // Deprecated, kept for backward compatibility
    pub timezone: Option<String>,
    pub max_daily_energy: Option<f64>,
    pub fetch_interval_seconds: Option<u64>,
}

impl HomeAssistantConfig {
    /// Parse history_duration (e.g., "180s", "3m", "1h") and return seconds
    /// Falls back to history_seconds if history_duration is not set
    pub fn get_history_seconds(&self) -> usize {
        // Try history_duration first
        if let Some(duration_str) = &self.history_duration {
            if let Some(seconds) = parse_duration(duration_str) {
                return seconds;
            }
        }

        // Fall back to history_seconds for backward compatibility
        self.history_seconds.unwrap_or(120)
    }

    /// Get formatted display string for history duration
    /// Returns user-defined format (e.g., "3m", "1h") or falls back to seconds format
    pub fn get_history_duration_display(&self) -> String {
        // If history_duration is set, format it for display
        if let Some(duration_str) = &self.history_duration {
            return format_duration_display(duration_str);
        }

        // Fall back to history_seconds format
        let seconds = self.history_seconds.unwrap_or(120);
        format!("{}s", seconds)
    }

    /// Get history time unit info for X-axis display
    /// Returns (value, unit_display, unit_label) tuple
    /// Examples: ("3", "min", "Time (min)"), ("1", "h", "Time (h)"), ("180", "s", "Time (s)")
    pub fn get_history_time_unit(&self) -> (f64, String, String) {
        // If history_duration is set, parse it
        if let Some(duration_str) = &self.history_duration {
            if let Some((value, unit)) = parse_duration_parts(duration_str) {
                let unit_display = match unit.to_lowercase().as_str() {
                    "s" | "sec" | "secs" | "second" | "seconds" => "s",
                    "m" | "min" | "mins" | "minute" | "minutes" => "min",
                    "h" | "hr" | "hrs" | "hour" | "hours" => "h",
                    _ => unit.as_str(),
                };
                let label = format!("Time ({})", unit_display);
                return (value, unit_display.to_string(), label);
            }
        }

        // Fall back to history_seconds
        let seconds = self.history_seconds.unwrap_or(120) as f64;
        (seconds, "s".to_string(), "Time (s)".to_string())
    }
}

/// Format duration string for display
/// Converts "3m" -> "3min", "1h" -> "1h", "180s" -> "180s"
fn format_duration_display(duration_str: &str) -> String {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return "120s".to_string();
    }

    // Find where the unit starts (first non-digit character)
    let split_pos = match duration_str
        .chars()
        .position(|c| !c.is_ascii_digit() && c != '.')
    {
        Some(pos) => pos,
        None => return duration_str.to_string(),
    };

    let (number_str, unit_str) = duration_str.split_at(split_pos);

    // Map unit to display format
    let display_unit = match unit_str.to_lowercase().as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => "s",
        "m" | "min" | "mins" | "minute" | "minutes" => "min",
        "h" | "hr" | "hrs" | "hour" | "hours" => "h",
        _ => unit_str,
    };

    format!("{}{}", number_str, display_unit)
}

/// Parse duration string like "180s", "3m", "1h" into seconds
fn parse_duration(duration_str: &str) -> Option<usize> {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return None;
    }

    // Find where the unit starts (first non-digit character)
    let split_pos = duration_str
        .chars()
        .position(|c| !c.is_ascii_digit() && c != '.')?;

    let (number_str, unit_str) = duration_str.split_at(split_pos);
    let number: f64 = number_str.parse().ok()?;

    let multiplier = match unit_str.to_lowercase().as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
        "m" | "min" | "mins" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3600.0,
        _ => return None,
    };

    Some((number * multiplier) as usize)
}

/// Parse duration string into (number, unit) parts
/// Examples: "3m" -> (3.0, "m"), "1h" -> (1.0, "h"), "180s" -> (180.0, "s")
fn parse_duration_parts(duration_str: &str) -> Option<(f64, String)> {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return None;
    }

    // Find where the unit starts (first non-digit character)
    let split_pos = duration_str
        .chars()
        .position(|c| !c.is_ascii_digit() && c != '.')?;

    let (number_str, unit_str) = duration_str.split_at(split_pos);
    let number: f64 = number_str.parse().ok()?;

    Some((number, unit_str.to_string()))
}

#[derive(Debug, Deserialize, Clone)]
pub struct Entities {
    // Core power sensors
    pub solar_production: String,
    pub grid_import: String,
    pub grid_export: String,
    pub load_consumption: String,
    pub load_current: Option<String>,

    // Battery sensors
    pub battery_power: Option<String>,
    pub battery_voltage: Option<String>,
    pub battery_soc: Option<String>,
    pub battery_temp: Option<String>,
    pub battery_current: Option<String>,

    // Inverter sensors
    pub inverter_temp: Option<String>,
    pub inverter_voltage: Option<String>,
    #[allow(dead_code)]
    pub inverter_status: Option<String>,

    // Grid sensors
    pub grid_voltage: Option<String>,
    pub grid_ct_power: Option<String>,
    pub inverter_frequency: Option<String>,

    // Power factor sensors
    pub load_power_factor: Option<String>,
    pub grid_power_factor: Option<String>,

    // Daily energy sensors
    pub day_battery_charge: Option<String>,
    pub day_battery_discharge: Option<String>,
    pub day_grid_import: Option<String>,
    pub day_grid_export: Option<String>,
    pub day_load_energy: Option<String>,
    pub day_pv_energy: Option<String>,

    // Total energy sensors
    pub total_pv_generation: Option<String>,
    pub remaining_solar: Option<String>,

    // Temperature sensors
    pub dc_transformer_temp: Option<String>,
    pub radiator_temp: Option<String>,

    // Essential power
    pub essential_power: Option<String>,

    // Statistics sensors
    pub all_time_energy_usage_peak: Option<String>,
    pub all_time_energy_usage_peak_date: Option<String>,
    pub all_time_load_peak: Option<String>,
    pub all_time_pv_power_peak: Option<String>,
    pub all_time_pv_power_peak_date: Option<String>,
    pub all_time_pv_yield_peak: Option<String>,
    pub all_time_pv_yield_peak_date: Option<String>,
    pub daily_pv_power_peak: Option<String>,
    pub daily_pv_power_peak_date: Option<String>,
    pub night_consume: Option<String>,
    pub pv_forecast_remain: Option<String>,
    pub pv_forecast_today: Option<String>,
    pub load_energy_yesterday: Option<String>,
    pub load_energy_total: Option<String>,
    pub day_consume: Option<String>,
}

fn get_config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Failed to determine home directory")?;

    let config_dir = PathBuf::from(home).join(".emon");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory at {:?}", config_dir))?;
    }

    Ok(config_dir)
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("config.toml"))
}

pub fn load_config() -> Result<Config> {
    load_config_at(None)
}

pub fn load_config_at(custom_path: Option<&Path>) -> Result<Config> {
    let path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => get_config_path()?,
    };
    let path_str = path.display().to_string();

    if !path.exists() {
        create_default_config(&path)?;
        anyhow::bail!(
            "Created default config at {}. Please edit it with your Home Assistant details.",
            path_str
        );
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file from {}", path_str))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file from {}", path_str))?;

    Ok(config)
}

fn create_default_config(path: &PathBuf) -> Result<()> {
    let default_config = r#"# Home Assistant Configuration
[home_assistant]
url = "http://homeassistant.local:8123"
token = "your_long_lived_access_token_here"
max_solar_power = 18000.0  # Maximum solar power in Watts (used for gauge scaling and history chart Y-axis)
battery_float_voltage = 54.0  # Battery floating voltage threshold (e.g., 54V for 48V battery system)
battery_capacity_kwh = 15.36  # Total battery capacity in kWh (e.g., 15.36 kWh for a 48V 320Ah system)
history_duration = "120s"  # History duration with unit: s/m/h (e.g., "180s", "3m", "1h", default: "120s")
timezone = "Asia/Bangkok"  # Timezone for display (default: Asia/Bangkok)
max_daily_energy = 100.0  # Maximum expected daily energy in kWh for bar chart scaling (default: 100.0)
fetch_interval_seconds = 5  # Data fetch interval in seconds (0 = realtime ~100ms, default: 5)

[home_assistant.entities]
# Core power sensors (required)
solar_production = "sensor.luxpower_sna_x_3_pv_power"
grid_import = "sensor.luxpower_sna_x_3_grid_power"
grid_export = "sensor.luxpower_sna_x_3_grid_power"
load_consumption = "sensor.total_load_power"

# Load current (optional)
load_current = "sensor.emon_load_energy_current"

# Battery sensors (optional)
battery_power = "sensor.luxpower_sna_x_3_battery_power"
battery_voltage = "sensor.luxpower_sna_x_3_inverter_1_battery_voltage"
battery_soc = "sensor.luxpower_sna_x_3_battery_state_of_charge"
battery_temp = "sensor.luxpower_sna_x_3_battery_temperature"
battery_current = "sensor.total_battery_current"

# Inverter sensors (optional)
inverter_temp = "sensor.luxpower_sna_x_3_inverter_1_temperature"
inverter_voltage = "sensor.luxpower_sna_x_3_inverter_1_ac_output_voltage"
inverter_status = "sensor.luxpower_sna_x_3_inverter_1_device_mode"
inverter_frequency = "sensor.luxpower_sna_x_3_inverter_1_ac_output_frequency"

# Grid sensors (optional)
grid_voltage = "sensor.luxpower_sna_x_3_grid_voltage"
grid_ct_power = "sensor.emon_grid_energy_power"

# Power factor sensors (optional)
load_power_factor = "sensor.emon_load_energy_factor"
grid_power_factor = "sensor.emon_grid_energy_factor"

# Daily energy sensors (optional)
day_battery_charge = "sensor.utility_pv_battery_daily_charge"
day_battery_discharge = "sensor.utility_pv_battery_daily_discharge"
day_grid_import = "sensor.utility_grid_daily_import"
day_grid_export = "sensor.utility_grid_daily_export"
day_load_energy = "sensor.emon_load_energy_today"
day_pv_energy = "sensor.utility_pv_daily_yield"

# Total energy sensors (optional)
total_pv_generation = "sensor.luxpower_sna_x_3_pv_energy"
remaining_solar = "sensor.energy_production_today_remaining"

# Temperature sensors (optional)
dc_transformer_temp = "sensor.dc_combiner_temperature"
radiator_temp = "sensor.pv_inverter_ac_temperature"

# Essential power (optional)
essential_power = "sensor.emon_load_energy_power"

# Statistics sensors (optional)
all_time_energy_usage_peak = "sensor.emon_glob_all_time_energy_usage_peak"
all_time_energy_usage_peak_date = "sensor.emon_glob_all_time_energy_usage_peak_date"
all_time_load_peak = "sensor.emon_glob_all_time_load_peak"
all_time_pv_power_peak = "sensor.emon_glob_all_time_pv_power_peak"
all_time_pv_power_peak_date = "sensor.emon_glob_all_time_pv_power_peak_date"
all_time_pv_yield_peak = "sensor.emon_glob_all_time_pv_yield_peak"
all_time_pv_yield_peak_date = "sensor.emon_glob_all_time_pv_yield_peak_date"
daily_pv_power_peak = "sensor.emon_glob_daily_pv_power_peak"
daily_pv_power_peak_date = "sensor.emon_glob_daily_pv_power_peak_date"
night_consume = "sensor.emon_glob_night_consume"
pv_forecast_remain = "sensor.emon_glob_pv_forecast_remain"
pv_forecast_today = "sensor.emon_glob_pv_forecast_today"
load_energy_yesterday = "sensor.emon_load_energy_yesterday"
load_energy_total = "sensor.emon_load_energy_total"
day_consume = "sensor.emon_glob_day_consume"
"#;

    fs::write(path, default_config)
        .with_context(|| format!("Failed to create default config at {}", path.display()))?;

    Ok(())
}
