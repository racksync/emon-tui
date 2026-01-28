<div align="center">

  <!-- Logo/Icon -->
  <pre>
   ______
  / ____/___  ____  ____  ____  ____  ____  ____  ______
 / /   / __ \/ __ \/ __ \/ __ \/ __ \/ __ \/ _ \/ ___/
/ /___/ /_/ / / / / / / / /_/ / / / / / / /  __/ /
\____/\____/_/ /_/_/ /_/\__, /_/_//_/_/ /_/\___/_/
                      /____/
  </pre>

  <!-- Badges -->
  [![Homebrew](https://img.shields.io/badge/dynamic/json?color=brightgreen&label=homebrew&prefix=v&query=%24.emon_tui.version&url=https%3A%2F%2Fraw.githubusercontent.com%2Fracksync%2Fhomebrew-emon-tui%2Fmain%2FFormula%2Femon-tui.rb)](https://github.com/racksync/homebrew-emon-tui)
  [![CI](https://github.com/racksync/emon-tui/actions/workflows/rust.yml/badge.svg)](https://github.com/racksync/emon-tui/actions/workflows/rust.yml)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

  <!-- Tagline -->
  **Real-time energy monitoring in your terminal**

  [Features](#-features) • [Install](#-installation) • [Config](#-configuration) • [Usage](#-usage)

</div>

---

## ✨ Features

- ⚡ **Real-time power monitoring** from Home Assistant entities
- 📊 **Visual gauges** with dynamic gradients for Solar, Load, and Battery
- 📈 **Line charts** showing power history with Braille markers
- 🔄 **Animated status** indicator with live updates
- 🔋 **Battery monitoring** - SOC, voltage, current, temperature
- 🌡️ **Temperature warnings** with dynamic color coding
- 📡 **Grid monitoring** - voltage, frequency, power factor
- 📅 **Daily energy tracking** - charge, discharge, import, export
- 📉 **Trend indicators** (↑ Rising, ↓ Falling, → Stable)
- ⏱️ **Configurable update interval** (realtime ~100ms or 1-10s)
- 🖥️ **Cross-platform** - macOS, Linux, Windows

---

## 📦 Installation

### 🍺 Homebrew (macOS/Linux)

```bash
brew tap racksync/homebrew-emon-tui
brew install emon-tui
```

### 🔧 Build from source

```bash
git clone https://github.com/racksync/emon-tui.git
cd emon-tui
cargo build --release
```

The binary will be at `target/release/emon`.

---

## ⚙️ Configuration

On first run, **emon** creates `~/.emon/config.toml`. Edit it with your Home Assistant details:

### Basic Setup

```toml
[home_assistant]
url = "http://homeassistant.local:8123"
token = "your_long_lived_access_token_here"

# System Settings
max_solar_power = 18000.0          # Max solar in Watts (for gauge scaling)
battery_float_voltage = 54.0       # Battery float voltage threshold
battery_capacity_kwh = 15.36       # Battery capacity in kWh
history_duration = "120s"          # History duration (s/m/h)
timezone = "Asia/Bangkok"          # Display timezone
max_daily_energy = 100.0           # Max daily energy for charts
fetch_interval_seconds = 5         # Update interval (0 = realtime)
```

### Sensor Entities

```toml
[home_assistant.entities]
# Core Power Sensors (Required)
solar_production = "sensor.solar_production"
grid_import = "sensor.grid_import"
grid_export = "sensor.grid_export"
load_consumption = "sensor.load_consumption"

# Optional: Battery, Inverter, Grid, Daily Energy, Statistics...
# See config.toml.example for all 62 sensors
```

### 🔑 Getting your Home Assistant Token

1. Open Home Assistant → User profile (bottom left)
2. Scroll to "Long-Lived Access Tokens"
3. Click "Create Token" → Name it "emon"
4. Copy token to `config.toml`

---

## 🚀 Usage

```bash
# Run
emon

# Show version
emon -v

# Custom config
emon -c /path/to/config.toml

# Help
emon -h
```

**Controls:** Press `q` to quit

---

## 🎨 Display Layout

```
┌─────────────────────────────────────────────────────────────┐
│  emon • Connected • 12:34:56 • 2s                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐ ┌─────────┐                        │
│  │  SOLAR  │ │  LOAD   │ │BATTERY  │                        │
│  │  4.2kW  │ │  2.1kW  │ │  85%    │                        │
│  └─────────┘ └─────────┘ └─────────┘                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐ ┌──────────────┐                         │
│  │   BATTERY    │ │     LOAD     │                         │
│  │ SOC: 85%     │ │ Power: 2.1kW │                         │
│  │ Volt: 52.4V  │ │ Volt: 230V   │                         │
│  └──────────────┘ └──────────────┘                         │
│  ┌──────────────┐ ┌──────────────┐                         │
│  │     GRID     │ │   INVERTER   │                         │
│  │ Volt: 230V   │ │ Status: ON   │                         │
│  │ Freq: 50Hz   │ │ Temp: 42°C   │                         │
│  └──────────────┘ └──────────────┘                         │
├─────────────────────────────────────────────────────────────┤
│  📊 Power History                      📊 Daily Energy       │
│  ████░░░░░░░░░░░░░░░░░░░░░░░           ▃▅▆█▆▅▃              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 Supported Sensors (62)

### 🔌 Core Power (required)
- Solar production • Grid import/export • Load consumption

### 🔋 Battery
- Power • Voltage • SOC • Temperature • Current

### 🔌 Inverter
- Temperature • Voltage • Status • Frequency

### 📡 Grid
- Voltage • CT power • Power factor

### 📊 Daily Energy
- Battery charge/discharge • Grid import/export • Load • PV

### 📈 Statistics
- All-time peaks • Daily peaks • PV forecast • Consumption totals

---

## 🎨 Color Coding

| Metric | Green | Yellow | Red |
|--------|-------|--------|-----|
| **Battery SOC** | >70% | 30-70% | <30% |
| **Voltage** | ≥220V (White) | 210-220V | <210V |
| **Temperature** | <35°C (White) | ≥35°C | - |
| **Current** | <80A (White) | 80-100A | ≥100A |

---

## 🔍 Troubleshooting

**No data showing?**
- Verify entity IDs match Home Assistant exactly
- Check sensors return numeric values (Watts)
- Press any key to dismiss error popup

**Connection issues?**
- Verify Home Assistant URL is accessible
- Check token is valid and not expired
- Ensure Home Assistant is running

**History chart empty?**
- Increase `history_duration` in config
- History is in-memory only (not persistent)

---

## 🛠️ Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Dependencies

- **ratatui** - TUI framework
- **tokio** - Async runtime
- **reqwest** - HTTP client
- **serde** - Serialization
- **chrono** - Date/time handling

---

## 📄 License

MIT © [racksync](https://github.com/racksync)

---

<div align="center">

  **⭐ Star us on GitHub — it helps!**

</div>
