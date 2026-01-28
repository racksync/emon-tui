# EMON-TUI - Missing Sensors

> Sensors published by Node-RED MQTT but **not yet supported** in Rust codebase (`src/config.rs`, `src/state.rs`, `src/ui.rs`)

---

## TOU (Time of Use) Sensors - 8 sensors

| Sensor Name | MQTT Topic | Description | Notes |
|-------------|-------------|-------------|--------|
| `on_peak_rate` | `emon/on_peak_rate` | On-peak electricity rate | Cost per kWh during peak hours |
| `off_peak_rate` | `emon/off_peak_rate` | Off-peak electricity rate | Cost per kWh during off-peak hours |
| `tou_daily_on_peak_kwh` | `emon/tou_daily_on_peak_kwh` | Daily on-peak consumption | kWh consumed during peak hours today |
| `tou_daily_off_peak_kwh` | `emon/tou_daily_off_peak_kwh` | Daily off-peak consumption | kWh consumed during off-peak hours today |
| `tou_daily_on_peak_cost` | `emon/tou_daily_on_peak_cost` | Daily on-peak cost | Total cost for peak hours today |
| `tou_daily_off_peak_cost` | `emon/tou_daily_off_peak_cost` | Daily off-peak cost | Total cost for off-peak hours today |
| `tou_status` | `emon/tou_status` | Current TOU status | "On Peak" or "Off Peak" |
| `pea_ft` | `emon/pea_ft` | PEA FT sensor | Electricity tariff/fee information |

**UI Integration Ideas:**
- Add TOU section showing current status and rates
- Daily breakdown of peak/off-peak consumption
- Cost analysis cards

---

## Inverter String Sensors - 12 sensors

> Per-string monitoring for multi-inverter systems

| Sensor Name | MQTT Topic | Description | Notes |
|-------------|-------------|-------------|--------|
| `emon_inverter_1_string_1_power` | `emon/emon_inverter_1_string_1_power` | Inverter 1 String 1 Power | Watts |
| `emon_inverter_1_string_1_voltage` | `emon/emon_inverter_1_string_1_voltage` | Inverter 1 String 1 Voltage | Volts |
| `emon_inverter_1_string_2_power` | `emon/emon_inverter_1_string_2_power` | Inverter 1 String 2 Power | Watts |
| `emon_inverter_1_string_2_voltage` | `emon/emon_inverter_1_string_2_voltage` | Inverter 1 String 2 Voltage | Volts |
| `emon_inverter_2_string_1_power` | `emon/emon_inverter_2_string_1_power` | Inverter 2 String 1 Power | Watts |
| `emon_inverter_2_string_1_voltage` | `emon/emon_inverter_2_string_1_voltage` | Inverter 2 String 1 Voltage | Volts |
| `emon_inverter_2_string_2_power` | `emon/emon_inverter_2_string_2_power` | Inverter 2 String 2 Power | Watts |
| `emon_inverter_2_string_2_voltage` | `emon/emon_inverter_2_string_2_voltage` | Inverter 2 String 2 Voltage | Volts |
| `emon_inverter_3_string_1_power` | `emon/emon_inverter_3_string_1_power` | Inverter 3 String 1 Power | Watts |
| `emon_inverter_3_string_1_voltage` | `emon/emon_inverter_3_string_1_voltage` | Inverter 3 String 1 Voltage | Volts |
| `emon_inverter_3_string_2_power` | `emon/emon_inverter_3_string_2_power` | Inverter 3 String 2 Power | Watts |
| `emon_inverter_3_string_2_voltage` | `emon/emon_inverter_3_string_2_voltage` | Inverter 3 String 2 Voltage | Volts |

**UI Integration Ideas:**
- Per-string power breakdown table
- Voltage balance indicator
- String efficiency comparison
- Mini gauges per string

---

## Cost Tracking Sensors - 9 sensors

| Sensor Name | MQTT Topic | Description | Notes |
|-------------|-------------|-------------|--------|
| `grid_daily_cost` | `emon/grid_daily_cost` | Daily grid electricity cost | Today's cost from grid |
| `grid_monthly_cost` | `emon/grid_monthly_cost` | Monthly grid electricity cost | This month's cost |
| `grid_yearly_cost` | `emon/grid_yearly_cost` | Yearly grid electricity cost | This year's cost |
| `total_daily_cost` | `emon/total_daily_cost` | Daily total electricity cost | Grid cost after solar savings |
| `total_daily_saved` | `emon/total_daily_saved` | Daily solar savings | Money saved from solar today |
| `total_monthly_cost` | `emon/total_monthly_cost` | Monthly total electricity cost | Grid cost after solar savings |
| `total_monthly_saved` | `emon/total_monthly_saved` | Monthly solar savings | Money saved from solar this month |
| `total_yearly_cost` | `emon/total_yearly_cost` | Yearly total electricity cost | Grid cost after solar savings |
| `total_yearly_saved` | `emon/total_yearly_saved` | Yearly solar savings | Money saved from solar this year |

**UI Integration Ideas:**
- Cost section in SUMMARY & STATISTICS
- Daily cost bar chart
- Monthly cost trend line chart
- Savings percentage display
- ROI calculator (solar investment)

---

## Additional Sensors - 5 sensors

| Sensor Name | MQTT Topic | Description | Notes |
|-------------|-------------|-------------|--------|
| `emon_load_ratio` | `emon/emon_load_ratio` | Load ratio | Percentage of max solar power used |
| `emon_night_ratio` | `emon/emon_night_ratio` | Night consumption ratio | Night/day consumption ratio |
| `emon_load_power` | `emon/emon_load_power` | Load power (alternative) | Alternative to `load_consumption` |
| `emon_voltage_load` | `emon/emon_voltage_load` | Load voltage | Voltage at load side |
| `emon_daily_battery_discharge_energy` | `emon/emon_daily_battery_discharge_energy` | Daily battery discharge | kWh discharged today |

**UI Integration Ideas:**
- `emon_load_ratio`: Already calculated in code, could display directly
- `emon_night_ratio`: Add to consumption stats
- `emon_load_power`: Could be used as alternative or for comparison
- `emon_voltage_load`: Add to LOAD card
- `emon_daily_battery_discharge_energy`: Combine with charge energy for net battery flow

---

## Summary

| Category | Count | Total Sensors |
|----------|-------|---------------|
| TOU Sensors | 8 | 8 |
| Inverter String Sensors | 12 | 20 |
| Cost Tracking Sensors | 9 | 29 |
| Additional Sensors | 5 | 34 |

**Total Missing Sensors: 34**

---

## Implementation Notes

### To Add These Sensors:

1. **Update `src/config.rs`**:
   - Add fields to `Entities` struct
   - Update default config template

2. **Update `src/state.rs`**:
   - Add fields to `PowerData` struct
   - Fetch sensors in `update()` method
   - Parse values with `parse_entity_value()`

3. **Update `src/ui.rs`**:
   - Add UI components (tables, cards, charts)
   - Integrate into layout
   - Style and format display

4. **Update `config.toml.example`**:
   - Add sensor examples with comments

### Priority Suggestions:

**High Priority:**
- Cost tracking sensors (9) - Financial insights
- TOU sensors (8) - Peak/off-peak optimization

**Medium Priority:**
- Inverter string sensors (12) - Technical monitoring
- Additional sensors (5) - Enhanced data

**Low Priority:**
- All are optional, can be added as needed

---

## Current Supported Sensors: 62

As of now, the Rust codebase supports:
- Core power: 4 (solar, grid import/export, load)
- Load: 3 (current, power factor, essential)
- Battery: 5 (power, voltage, soc, temp, current)
- Inverter: 6 (temp, voltage, status, frequency, dc temp, radiator temp)
- Grid: 3 (voltage, ct power, power factor)
- Daily energy: 6 (battery charge/discharge, grid import/export, load, pv)
- Total energy: 2 (pv generation, remaining)
- Statistics: 13 (peaks, consumption, forecast, load totals)
- **Total: 62 sensors**

**With Missing Sensors Added: 96 sensors total**

---

*Document created: 2026-01-23*
*Last updated: 2026-01-23*
