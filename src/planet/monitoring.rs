use super::*;

pub fn monitor(planet: &mut Planet, params: &Params) {
    if planet.cycles % params.monitoring.interval_cycles != 1 {
        return;
    }

    planet.msgs.remove_outdated(planet.cycles);

    // Temperature warnings
    if planet.stat.average_air_temp > params.monitoring.warn_high_temp_threshold {
        planet.msgs.append_persitent_warn(Msg {
            cycles: planet.cycles,
            kind: MsgKind::WarnHighTemp,
            span: None,
        });
    } else {
        planet.msgs.remove_persitent_warn(&MsgKind::WarnHighTemp);
    }

    if planet.stat.average_air_temp < params.monitoring.warn_low_temp_threshold {
        planet.msgs.append_persitent_warn(Msg {
            cycles: planet.cycles,
            kind: MsgKind::WarnLowTemp,
            span: None,
        });
    } else {
        planet.msgs.remove_persitent_warn(&MsgKind::WarnLowTemp);
    }

    // Atmosphere warnings
    if planet.atmo.partial_pressure(GasKind::Oxygen) < params.monitoring.warn_low_oxygen_threshold {
        planet.msgs.append_persitent_warn(Msg {
            cycles: planet.cycles,
            kind: MsgKind::WarnLowOxygen,
            span: None,
        });
    } else {
        planet.msgs.remove_persitent_warn(&MsgKind::WarnLowOxygen);
    }
}
