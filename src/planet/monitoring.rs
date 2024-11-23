use super::*;

pub fn monitor(planet: &mut Planet, params: &Params) {
    if planet.cycles % params.monitoring.interval_cycles != 1 {
        return;
    }

    planet.msgs.remove_outdated(planet.cycles);

    if planet.stat.average_air_temp > params.monitoring.warn_high_temp_threshold {
        planet.msgs.append_temp(Msg {
            cycles: planet.cycles,
            kind: MsgKind::WarnHighTemp,
            span: None,
        });
    } else {
        planet.msgs.remove_temp(&MsgKind::WarnHighTemp);
    }

    if planet.stat.average_air_temp < params.monitoring.warn_low_temp_threshold {
        planet.msgs.append_temp(Msg {
            cycles: planet.cycles,
            kind: MsgKind::WarnLowTemp,
            span: None,
        });
    } else {
        planet.msgs.remove_temp(&MsgKind::WarnLowTemp);
    }
}
