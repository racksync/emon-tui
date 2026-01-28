use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;

use crate::config::Config;
use crate::homeassistant::{EntityState, HomeAssistant};

#[derive(Debug, Clone)]
pub struct PowerData {
    // Core power readings
    pub solar: f64,
    pub grid_import: f64,
    pub grid_export: f64,
    pub load: f64,
    pub load_current: f64,
    
    // Battery readings
    pub battery_power: f64,
    pub battery_voltage: f64,
    pub battery_soc: f64,
    pub battery_temp: f64,
    pub battery_current: f64,
    
    // Inverter readings
    pub inverter_temp: f64,
    pub inverter_voltage: f64,
    pub inverter_frequency: f64,
    pub inverter_status: String,
    
    // Grid readings
    pub grid_voltage: f64,
    #[allow(dead_code)]
    pub grid_ct_power: f64,
    
    // Power factor
    pub load_power_factor: f64,
    pub grid_power_factor: f64,
    
    // Daily energy totals
    pub day_battery_charge: f64,
    pub day_battery_discharge: f64,
    pub day_grid_import: f64,
    pub day_grid_export: f64,
    pub day_load_energy: f64,
    pub day_pv_energy: f64,
    
    // Total energy
    #[allow(dead_code)]
    pub total_pv_generation: f64,
    pub remaining_solar: f64,
    
    // Additional temperatures
    pub dc_transformer_temp: f64,
    pub radiator_temp: f64,
    
    // Essential power
    #[allow(dead_code)]
    pub essential_power: f64,
    
    // Statistics
    pub all_time_energy_usage_peak: f64,
    pub all_time_energy_usage_peak_date: String,
    pub all_time_load_peak: f64,
    pub all_time_pv_power_peak: f64,
    pub all_time_pv_power_peak_date: String,
    pub all_time_pv_yield_peak: f64,
    pub all_time_pv_yield_peak_date: String,
    pub daily_pv_power_peak: f64,
    pub daily_pv_power_peak_date: String,
    pub load_ratio: f64,
    pub night_consume: f64,
    pub pv_forecast_remain: f64,
    pub pv_forecast_today: f64,
    pub load_energy_yesterday: f64,
    pub load_energy_total: f64,
    pub day_consume: f64,
    
    #[allow(dead_code)]
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub config_path: Option<PathBuf>,
    pub ha: HomeAssistant,
    pub history: Vec<PowerData>,
    pub last_fetch: Option<Instant>,
    pub error: Option<String>,
    pub max_values: PowerData,
}

impl AppState {
    pub fn new(config: Config, config_path: Option<PathBuf>) -> Self {
        let ha = HomeAssistant::new(
            config.home_assistant.url.clone(),
            config.home_assistant.token.clone(),
        );

        let history_size = config.home_assistant.get_history_seconds();

        Self {
            config,
            config_path,
            ha,
            history: Vec::with_capacity(history_size),
            last_fetch: None,
            error: None,
            max_values: PowerData {
                solar: 0.0,
                grid_import: 0.0,
                grid_export: 0.0,
                load: 0.0,
                load_current: 0.0,
                battery_power: 0.0,
                battery_voltage: 0.0,
                battery_soc: 0.0,
                battery_temp: 0.0,
                battery_current: 0.0,
                inverter_temp: 0.0,
                inverter_voltage: 0.0,
                inverter_frequency: 0.0,
                grid_voltage: 0.0,
                grid_ct_power: 0.0,
                load_power_factor: 0.0,
                grid_power_factor: 0.0,
                day_battery_charge: 0.0,
                day_battery_discharge: 0.0,
                day_grid_import: 0.0,
                day_grid_export: 0.0,
                day_load_energy: 0.0,
                day_pv_energy: 0.0,
                total_pv_generation: 0.0,
                remaining_solar: 0.0,
                dc_transformer_temp: 0.0,
                radiator_temp: 0.0,
                essential_power: 0.0,
                all_time_energy_usage_peak: 0.0,
                all_time_energy_usage_peak_date: String::new(),
                all_time_load_peak: 0.0,
                all_time_pv_power_peak: 0.0,
                all_time_pv_power_peak_date: String::new(),
                all_time_pv_yield_peak: 0.0,
                all_time_pv_yield_peak_date: String::new(),
                daily_pv_power_peak: 0.0,
                daily_pv_power_peak_date: String::new(),
                load_ratio: 0.0,
                night_consume: 0.0,
                pv_forecast_remain: 0.0,
                pv_forecast_today: 0.0,
                load_energy_yesterday: 0.0,
                load_energy_total: 0.0,
                day_consume: 0.0,
                inverter_status: String::from("Unknown"),
                timestamp: Instant::now(),
            },
        }
    }

    pub async fn update(&mut self) -> Result<()> {
        // Core power sensors
        let solar = self.ha.get_state(&self.config.home_assistant.entities.solar_production).await;
        let grid_import = self.ha.get_state(&self.config.home_assistant.entities.grid_import).await;
        let grid_export = self.ha.get_state(&self.config.home_assistant.entities.grid_export).await;
        let load = self.ha.get_state(&self.config.home_assistant.entities.load_consumption).await;
        
        // Load current (optional)
        let load_current = match &self.config.home_assistant.entities.load_current {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };

        // Battery sensors
        let bat_power = match &self.config.home_assistant.entities.battery_power {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let bat_voltage = match &self.config.home_assistant.entities.battery_voltage {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let bat_soc = match &self.config.home_assistant.entities.battery_soc {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let bat_temp = match &self.config.home_assistant.entities.battery_temp {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let bat_current = match &self.config.home_assistant.entities.battery_current {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Inverter sensors
        let inv_temp = match &self.config.home_assistant.entities.inverter_temp {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let inv_voltage = match &self.config.home_assistant.entities.inverter_voltage {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let inv_freq = match &self.config.home_assistant.entities.inverter_frequency {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let inv_status = match &self.config.home_assistant.entities.inverter_status {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Grid sensors
        let grid_voltage = match &self.config.home_assistant.entities.grid_voltage {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let grid_ct_power = match &self.config.home_assistant.entities.grid_ct_power {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Power factor sensors
        let load_power_factor = match &self.config.home_assistant.entities.load_power_factor {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let grid_power_factor = match &self.config.home_assistant.entities.grid_power_factor {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Daily energy sensors
        let day_bat_charge = match &self.config.home_assistant.entities.day_battery_charge {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let day_bat_discharge = match &self.config.home_assistant.entities.day_battery_discharge {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let day_grid_imp = match &self.config.home_assistant.entities.day_grid_import {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let day_grid_exp = match &self.config.home_assistant.entities.day_grid_export {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let day_load_ene = match &self.config.home_assistant.entities.day_load_energy {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let day_pv_ene = match &self.config.home_assistant.entities.day_pv_energy {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Total energy sensors
        let total_pv_gen = match &self.config.home_assistant.entities.total_pv_generation {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let remaining_sol = match &self.config.home_assistant.entities.remaining_solar {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Temperature sensors
        let dc_trans_temp = match &self.config.home_assistant.entities.dc_transformer_temp {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let rad_temp = match &self.config.home_assistant.entities.radiator_temp {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        
        // Essential power
        let ess_power = match &self.config.home_assistant.entities.essential_power {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };

        // Statistics sensors
        let stat_energy_usage_peak = match &self.config.home_assistant.entities.all_time_energy_usage_peak {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_energy_usage_peak_date = match &self.config.home_assistant.entities.all_time_energy_usage_peak_date {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_load_peak = match &self.config.home_assistant.entities.all_time_load_peak {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_pv_power_peak = match &self.config.home_assistant.entities.all_time_pv_power_peak {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_pv_power_peak_date = match &self.config.home_assistant.entities.all_time_pv_power_peak_date {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_pv_yield_peak = match &self.config.home_assistant.entities.all_time_pv_yield_peak {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_pv_yield_peak_date = match &self.config.home_assistant.entities.all_time_pv_yield_peak_date {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_daily_pv_power_peak = match &self.config.home_assistant.entities.daily_pv_power_peak {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_daily_pv_power_peak_date = match &self.config.home_assistant.entities.daily_pv_power_peak_date {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_night_consume = match &self.config.home_assistant.entities.night_consume {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_pv_forecast_remain = match &self.config.home_assistant.entities.pv_forecast_remain {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_pv_forecast_today = match &self.config.home_assistant.entities.pv_forecast_today {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_load_energy_yesterday = match &self.config.home_assistant.entities.load_energy_yesterday {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_load_energy_total = match &self.config.home_assistant.entities.load_energy_total {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };
        let stat_day_consume = match &self.config.home_assistant.entities.day_consume {
            Some(entity) => self.ha.get_state(entity).await.ok(),
            None => None,
        };

        let solar_val = solar.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0);
        let grid_import_val = grid_import.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0);
        let grid_export_val = grid_export.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0);
        let load_val = load.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0);

        let data = PowerData {
            solar: solar_val,
            grid_import: grid_import_val,
            grid_export: grid_export_val,
            load: load_val,
            load_current: load_current.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            battery_power: bat_power.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            battery_voltage: bat_voltage.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            battery_soc: bat_soc.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            battery_temp: bat_temp.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            battery_current: bat_current.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            inverter_temp: inv_temp.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            inverter_voltage: inv_voltage.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            inverter_frequency: inv_freq.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            inverter_status: inv_status.map(|e| e.state.clone()).unwrap_or_else(|| "Unknown".to_string()),
            grid_voltage: grid_voltage.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            grid_ct_power: grid_ct_power.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            load_power_factor: load_power_factor.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            grid_power_factor: grid_power_factor.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_battery_charge: day_bat_charge.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_battery_discharge: day_bat_discharge.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_grid_import: day_grid_imp.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_grid_export: day_grid_exp.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_load_energy: day_load_ene.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_pv_energy: day_pv_ene.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            total_pv_generation: total_pv_gen.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            remaining_solar: remaining_sol.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            dc_transformer_temp: dc_trans_temp.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            radiator_temp: rad_temp.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            essential_power: ess_power.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            all_time_energy_usage_peak: stat_energy_usage_peak.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            all_time_energy_usage_peak_date: stat_energy_usage_peak_date.map(|e| e.state.clone()).unwrap_or_default(),
            all_time_load_peak: stat_load_peak.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            all_time_pv_power_peak: stat_pv_power_peak.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            all_time_pv_power_peak_date: stat_pv_power_peak_date.map(|e| e.state.clone()).unwrap_or_default(),
            all_time_pv_yield_peak: stat_pv_yield_peak.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            all_time_pv_yield_peak_date: stat_pv_yield_peak_date.map(|e| e.state.clone()).unwrap_or_default(),
            daily_pv_power_peak: stat_daily_pv_power_peak.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            daily_pv_power_peak_date: stat_daily_pv_power_peak_date.map(|e| e.state.clone()).unwrap_or_default(),
            load_ratio: {
                let max_power_w = self.config.home_assistant.max_solar_power.unwrap_or(18000.0);
                if max_power_w > 0.0 {
                    (load_val / max_power_w) * 100.0
                } else {
                    0.0
                }
            },
            night_consume: stat_night_consume.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            pv_forecast_remain: stat_pv_forecast_remain.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            pv_forecast_today: stat_pv_forecast_today.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            load_energy_yesterday: stat_load_energy_yesterday.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            load_energy_total: stat_load_energy_total.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            day_consume: stat_day_consume.map(|e| self.parse_entity_value(&e)).unwrap_or(0.0),
            timestamp: Instant::now(),
        };

        self.update_max_values(&data);
        self.history.push(data);

        let history_size = self.config.home_assistant.get_history_seconds();
        if self.history.len() > history_size {
            self.history.remove(0);
        }

        self.last_fetch = Some(Instant::now());
        self.error = None;

        Ok(())
    }

    fn parse_entity_value(&self, entity: &EntityState) -> f64 {
        entity.state.parse::<f64>().unwrap_or(0.0)
    }

    fn update_max_values(&mut self, data: &PowerData) {
        self.max_values.solar = self.max_values.solar.max(data.solar);
        self.max_values.grid_import = self.max_values.grid_import.max(data.grid_import);
        self.max_values.grid_export = self.max_values.grid_export.max(data.grid_export);
        self.max_values.load = self.max_values.load.max(data.load);
    }

    pub fn get_solar_history(&self) -> Vec<f64> {
        self.history.iter().map(|d| d.solar).collect()
    }

    pub fn get_history_seconds(&self) -> usize {
        self.config.home_assistant.get_history_seconds()
    }

    pub fn get_history_duration_display(&self) -> String {
        self.config.home_assistant.get_history_duration_display()
    }

    pub fn get_history_time_unit(&self) -> (f64, String, String) {
        self.config.home_assistant.get_history_time_unit()
    }

    pub fn get_load_history(&self) -> Vec<f64> {
        self.history.iter().map(|d| d.load).collect()
    }

    #[allow(dead_code)]
    pub fn get_grid_import_history(&self) -> Vec<f64> {
        self.history.iter().map(|d| d.grid_import).collect()
    }

    #[allow(dead_code)]
    pub fn get_grid_export_history(&self) -> Vec<f64> {
        self.history.iter().map(|d| d.grid_export).collect()
    }

    #[allow(dead_code)]
    pub fn get_battery_power_history(&self) -> Vec<f64> {
        self.history.iter().map(|d| d.battery_power).collect()
    }
}
