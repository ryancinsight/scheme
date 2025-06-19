use crate::cfd::flow_rate::calculate_channel_flow_rates;
use crate::cfd::hydrodynamic_resistance::calculate_all_resistances;
use crate::cfd::pressure::calculate_node_pressures;
use crate::config::CfdConfig;
use crate::error::SimulationError;
use crate::geometry::{CfdResults, ChannelSystem};

pub fn run_simulation(
    system: &ChannelSystem,
    config: &CfdConfig,
) -> Result<CfdResults, SimulationError> {
    let channel_resistances = calculate_all_resistances(system, config);
    let node_pressures = calculate_node_pressures(system, &channel_resistances, config)?;
    let channel_flow_rates =
        calculate_channel_flow_rates(system, &node_pressures, &channel_resistances);

    Ok(CfdResults {
        system: system.clone(),
        node_pressures,
        channel_flow_rates,
        channel_resistances,
    })
} 