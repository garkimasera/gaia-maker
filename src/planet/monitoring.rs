use super::*;

pub fn monitor(planet: &mut Planet, params: &Params) {
    if planet.cycles % params.monitoring.interval_cycles != 1 {
        return;
    }

    planet.reports.remove_outdated(planet.cycles);

    // Temperature warnings
    if planet.stat.average_air_temp > params.monitoring.warn_high_temp_threshold {
        planet
            .reports
            .append_persitent_warn(planet.cycles, ReportContent::WarnHighTemp);
    } else {
        planet
            .reports
            .remove_persitent_warn(&ReportContent::WarnHighTemp);
    }

    if planet.stat.average_air_temp < params.monitoring.warn_low_temp_threshold {
        planet
            .reports
            .append_persitent_warn(planet.cycles, ReportContent::WarnLowTemp);
    } else {
        planet
            .reports
            .remove_persitent_warn(&ReportContent::WarnLowTemp);
    }

    // Atmosphere warnings
    if planet.atmo.partial_pressure(GasKind::Oxygen) < params.monitoring.warn_low_oxygen_threshold {
        planet
            .reports
            .append_persitent_warn(planet.cycles, ReportContent::WarnLowOxygen);
    } else {
        planet
            .reports
            .remove_persitent_warn(&ReportContent::WarnLowOxygen);
    }

    if planet.atmo.partial_pressure(GasKind::CarbonDioxide)
        < params.monitoring.warn_low_carbon_dioxide_threshold
    {
        planet
            .reports
            .append_persitent_warn(planet.cycles, ReportContent::WarnLowCarbonDioxide);
    } else {
        planet
            .reports
            .remove_persitent_warn(&ReportContent::WarnLowCarbonDioxide);
    }
}
