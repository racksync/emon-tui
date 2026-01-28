# emon-tui

[![Homebrew](https://img.shields.io/badge/dynamic/json?color=brightgreen&label=homebrew&prefix=v&query=%24.emon_tui.version&url=https%3A%2F%2Fraw.githubusercontent.com%2Fracksync%2Fhomebrew-emon-tui%2Fmain%2FFormula%2Femon-tui.rb)](https://github.com/racksync/homebrew-emon-tui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A terminal UI (TUI) application for monitoring real-time energy data from Home Assistant. Displays solar production, grid import/export, and load consumption with gauges, charts, and data tables.

## Features

- Real-time power monitoring from Home Assistant entities
- Visual gauges for Solar Production, Load Consumption, and Battery SOC with dynamic gradients
- Animated status indicator with live updates (spinning)
- Data tables for power flow and system status (2x2 grid layout)
- Line charts showing power history (configurable duration) with Braille markers
- Trend indicators (↑ Rising, ↓ Falling, → Stable)
- Configurable update interval (0-10 seconds, or realtime ~100ms)
- Cross-platform support (macOS, WSL, Linux)
- Battery power status (Charging/Discharging/Idle) with float indicator
- Temperature monitoring (Battery, Inverter) with dynamic color warnings
- Grid voltage, frequency, and power factor display
- Color-coded battery SOC (Green >70%, Yellow 30-70%, Red <30%)
- Dynamic voltage warning system (White ≥220V, Orange 210-220V, Red <210V)
- Configurable max solar power for accurate gauge scaling (default: 18000W / 18kW)
- Day/Night consumption tracking with ratio display
- All-time peak tracking (PV power, PV yield, Load, Energy usage) with timestamps
- PV forecast display (today's production + remaining)
- Load energy statistics (yesterday, total all-time, daily)

## Supported Sensors (62)

emon-tui currently supports **62 sensors** organized into categories:

### Core Power Sensors (4)
- Solar production
- Grid import
- Grid export
- Load consumption

### Load Sensors (3)
- Load current
- Load power factor
- Essential power

### Battery Sensors (5)
- Battery power
- Battery voltage
- Battery state of charge (SOC)
- Battery temperature
- Battery current

### Inverter Sensors (6)
- Inverter temperature
- Inverter voltage
- Inverter status
- Inverter frequency
- DC transformer temperature
- Radiator temperature

### Grid Sensors (3)
- Grid voltage
- Grid CT power
- Grid power factor

### Daily Energy Sensors (6)
- Day battery charge
- Day battery discharge
- Day grid import
- Day grid export
- Day load energy
- Day PV energy

### Total Energy Sensors (2)
- Total PV generation
- Remaining solar forecast

### Statistics Sensors (13)
- All-time energy usage peak
- All-time energy usage peak date
- All-time load peak
- All-time PV power peak
- All-time PV power peak date
- All-time PV yield peak
- All-time PV yield peak date
- Daily PV power peak
- Daily PV power peak date
- Night consumption
- PV forecast remaining
- PV forecast today
- Load energy yesterday
- Load energy total all-time
- Day consumption

## Roadmap / Planned Features

Additional sensors available from Node-RED MQTT (not yet integrated):

### TOU Sensors (8)
- Peak/off-peak rates and consumption
- Peak/off-peak daily costs
- Current TOU status (On Peak / Off Peak)

### Inverter String Sensors (12)
- Per-string power and voltage for multi-inverter systems
- 3 inverters × 2 strings each = 6 strings total

### Cost Tracking Sensors (9)
- Grid costs: daily, monthly, yearly
- Total costs: daily, monthly, yearly
- Solar savings: daily, monthly, yearly

### Additional Sensors (5)
- Load ratios and night/day consumption ratios
- Alternative load power and voltage sensors
- Battery discharge energy tracking

See `MISSING_SENSORS.md` for complete list with MQTT topics and integration details.

## Prerequisites

- Home Assistant instance running and accessible
- Home Assistant Long-Lived Access Token
- Power sensor entities in Home Assistant returning numeric values (in Watts)

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap racksync/homebrew-emon-tui
brew install emon-tui
```

### Build from source

```bash
git clone <repository-url>
cd emon-tui/emon-tui
cargo build --release
```

The binary will be available at `target/release/emon-tui`.

## Configuration

On first run, emon will create a `config.toml` file at `~/.emon/config.toml`. Edit it with your Home Assistant details:

### Basic Configuration

```toml
# Home Assistant Configuration
[home_assistant]
url = "http://homeassistant.local:8123"
token = "your_long_lived_access_token_here"

# System Settings
max_solar_power = 18000.0              # Maximum solar power in Watts
battery_float_voltage = 54.0              # Battery floating voltage threshold
battery_capacity_kwh = 15.36            # Battery capacity in kWh
history_duration = "120s"               # History duration (s/m/h)
timezone = "Asia/Bangkok"               # Timezone for display
max_daily_energy = 100.0               # Max daily energy for charts
fetch_interval_seconds = 5               # Update interval (0 = realtime)
```

### Sensor Entity Configuration

```toml
[home_assistant.entities]
# Core Power Sensors (Required)
solar_production = "sensor.solar_production"
grid_import = "sensor.grid_import"
grid_export = "sensor.grid_export"
load_consumption = "sensor.load_consumption"

# Load Sensors (Optional)
load_current = "sensor.emon_load_energy_current"
load_power_factor = "sensor.emon_load_energy_factor"

# Battery Sensors (Optional)
battery_power = "sensor.battery_power"
battery_voltage = "sensor.battery_voltage"
battery_soc = "sensor.battery_soc"
battery_temp = "sensor.battery_temp"
battery_current = "sensor.battery_current"

# Inverter Sensors (Optional)
inverter_temp = "sensor.inverter_temp"
inverter_voltage = "sensor.inverter_voltage"
inverter_status = "sensor.inverter_status"
inverter_frequency = "sensor.inverter_frequency"
dc_transformer_temp = "sensor.dc_transformer_temp"
radiator_temp = "sensor.radiator_temp"

# Grid Sensors (Optional)
grid_voltage = "sensor.grid_voltage"
grid_ct_power = "sensor.grid_ct_power"
grid_power_factor = "sensor.emon_grid_energy_factor"

# Daily Energy Sensors (Optional)
day_battery_charge = "sensor.day_battery_charge"
day_battery_discharge = "sensor.day_battery_discharge"
day_grid_import = "sensor.day_grid_import"
day_grid_export = "sensor.day_grid_export"
day_load_energy = "sensor.emon_load_energy_today"
day_pv_energy = "sensor.day_pv_energy"

# Total Energy Sensors (Optional)
total_pv_generation = "sensor.total_pv_generation"
remaining_solar = "sensor.remaining_solar"

# Statistics Sensors (Optional)
all_time_energy_usage_peak = "sensor.all_time_energy_usage_peak"
all_time_energy_usage_peak_date = "sensor.all_time_energy_usage_peak_date"
all_time_load_peak = "sensor.all_time_load_peak"
all_time_pv_power_peak = "sensor.all_time_pv_power_peak"
all_time_pv_power_peak_date = "sensor.all_time_pv_power_peak_date"
all_time_pv_yield_peak = "sensor.all_time_pv_yield_peak"
all_time_pv_yield_peak_date = "sensor.all_time_pv_yield_peak_date"
daily_pv_power_peak = "sensor.daily_pv_power_peak"
daily_pv_power_peak_date = "sensor.daily_pv_power_peak_date"
night_consume = "sensor.night_consume"
pv_forecast_remain = "sensor.pv_forecast_remain"
pv_forecast_today = "sensor.pv_forecast_today"
load_energy_yesterday = "sensor.emon_load_energy_yesterday"
load_energy_total = "sensor.emon_load_energy_total"
day_consume = "sensor.emon_glob_day_consume"
```

See `config.toml.example` for the complete template with all 62 sensors organized by category.

### Getting your Home Assistant Token

1. Open Home Assistant
2. Go to your user profile (bottom left)
3. Scroll down to "Long-Lived Access Tokens"
4. Click "Create Token"
5. Give it a name (e.g., "emon-tui")
6. Copy the token and paste it into `config.toml`

### Finding your Entity IDs

1. Go to Home Assistant Settings → Devices & Services → Entities
2. Search for your power sensors
3. Use the entity ID (e.g., `sensor.solar_power`)

## Usage

Run the application:

```bash
emon
```

Or directly with cargo:

```bash
cargo run --release
```

### Command-Line Options

```bash
emon [OPTIONS]
```

| Flag | Long Form | Description |
|-------|-----------|-------------|
| `-h` | `--help` | Print help information |
| `-v` | `--version` | Print version information |
| `-c` | `--config <CONFIG>` | Path to custom config file |

#### Examples

Show version:
```bash
emon -v
# or
emon --version
```

Show help:
```bash
emon -h
# or
emon --help
```

Run with custom config:
```bash
emon -c /path/to/custom-config.toml
# or
emon --config /path/to/custom-config.toml
```

### Controls

- Press `q` to quit

### Display Layout

The TUI shows:

1. **Header**: App name, connection status, current time, update timer
2. **Gauges** (9% height):
   - Solar Production (yellow gradient)
   - Load Consumption (blue gradient)
   - Battery SOC (color-coded: green/yellow/red)
3. **Data Tables** (44% height) - 2x2 Grid:
   - **BATTERY** (top-left): SOC, voltage, current, temperature
   - **LOAD** (top-right): Power, voltage, current, power factor
   - **GRID** (bottom-left): Voltage, frequency, power factor
   - **INVERTER** (bottom-right): Status, temperatures
4. **Statistics Table**: Summary of all-time peaks, daily stats, forecasts, consumption
5. **Charts** (43% height):
   - Power history line chart (solar, load, battery)
   - Daily energy bar chart
6. **Footer**: Status information

### Color Coding

**Temperature:**
- White: <35°C
- Orange: ≥35°C

**Voltage:**
- White: ≥220V
- Orange: 210-220V
- Red: <210V

**Current:**
- White: <80A
- Orange: 80-100A
- Red: ≥100A

**Battery SOC:**
- Green: >70%
- Yellow: 30-70%
- Red: <30%

### Status Indicators

- **LIVE** (green): Last update <5 seconds ago
- **SYNC** (yellow): Last update 5-10 seconds ago
- **STALE** (gray): Last update >10 seconds ago
- **ERROR** (red): Connection or data fetch error

## Troubleshooting

### Connection Issues

- Verify your Home Assistant URL is correct and accessible
- Check that your token is valid and not expired
- Ensure Home Assistant is running

### No Data Showing

- Verify entity IDs match exactly what's in Home Assistant
- Check that your sensors return numeric values (Watts)
- Look at the status area for error messages
- Press any key to dismiss error popup

### Gauge Shows 0% Even With Data

- Gauges scale relative to maximum value seen since the app started
- This ensures gauge is always visible even at low power levels
- Values will normalize over time

### History Chart Not Showing

- Check `history_duration` setting in config (default: "120s")
- Increase if you want longer history (e.g., "5m", "15m", "1h")
- History is stored in memory only (not persistent)

## Data Storage

emon-tui stores data **in memory only** (no disk/database):

- Storage type: In-memory circular buffer (Vec<PowerData>)
- Buffer size: Determined by `history_duration` and `fetch_interval_seconds`
- Example: 15 minutes history at 2-second interval = 450 data points
- Memory usage: ~900KB for 450 entries
- Persistence: None - data is lost when app exits

## Development

### Build in Debug Mode

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs          (89 lines)   - Entry point & main loop
├── config.rs        (334 lines)  - Configuration management
├── state.rs         (465 lines)  - Application state & data fetching
├── homeassistant.rs (65 lines)   - Home Assistant API client
└── ui.rs           (1352 lines)  - TUI rendering
```

### Dependencies

- **ratatui**: TUI framework
- **crossterm**: Terminal handling
- **tokio**: Async runtime
- **reqwest**: HTTP client
- **serde**: Serialization
- **chrono**: Date/time handling
- **chrono-tz**: Timezone support
- **toml**: Config parsing
- **anyhow**: Error handling

## Node-RED Integration

A Node-RED flow is provided to publish sensor values to MQTT for emon-tui to consume:

- **File**: `node-red-mqtt-publisher.json`
- **Trigger**: Every 2 seconds
- **Pattern**: Read from global context → Publish to MQTT topic `emon/{sensor_name}`
- **Sensors**: Publishes 57+ sensors from global context

Import the flow into Node-RED:
1. Open Node-RED
2. Menu → Import
3. Select `node-red-mqtt-publisher.json`
4. Deploy

**Note**: emon-tui currently uses Home Assistant REST API directly. MQTT integration is available but requires switching the data source implementation.

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

### Areas for Contribution

- Add support for roadmap sensors (TOU, inverter strings, cost tracking)
- MQTT data source implementation (alternative to Home Assistant API)
- Data persistence (SQLite, InfluxDB, etc.)
- Additional visualizations
- Internationalization (i18n)

## Related Files

- `config.toml.example` - Complete configuration template
- `MISSING_SENSORS.md` - Detailed list of 34 unimplemented sensors
- `node-red-mqtt-publisher.json` - Node-RED MQTT publisher flow
