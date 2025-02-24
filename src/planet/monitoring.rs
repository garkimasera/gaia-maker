use super::*;

pub fn monitor(planet: &mut Planet, params: &Params) {
    if planet.cycles % params.monitoring.interval_cycles != 1 {
        return;
    }

    planet.msgs.remove_outdated(planet.cycles);

    // Temperature warnings
    if planet.stat.average_air_temp > params.monitoring.warn_high_temp_threshold {
        planet
            .msgs
            .append_persitent_warn(planet.cycles, MsgContent::WarnHighTemp);
    } else {
        planet.msgs.remove_persitent_warn(&MsgContent::WarnHighTemp);
    }

    if planet.stat.average_air_temp < params.monitoring.warn_low_temp_threshold {
        planet
            .msgs
            .append_persitent_warn(planet.cycles, MsgContent::WarnLowTemp);
    } else {
        planet.msgs.remove_persitent_warn(&MsgContent::WarnLowTemp);
    }

    // Atmosphere warnings
    if planet.atmo.partial_pressure(GasKind::Oxygen) < params.monitoring.warn_low_oxygen_threshold {
        planet
            .msgs
            .append_persitent_warn(planet.cycles, MsgContent::WarnLowOxygen);
    } else {
        planet
            .msgs
            .remove_persitent_warn(&MsgContent::WarnLowOxygen);
    }
}
