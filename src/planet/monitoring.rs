use super::*;

impl Planet {
    pub fn monitor(&mut self, params: &Params, report_span: u64) {
        if self.cycles % params.monitoring.interval_cycles != 1 {
            return;
        }

        self.reports.remove_outdated(self.cycles, report_span);

        // Temperature warnings
        if self.stat.average_air_temp > params.monitoring.warn_high_temp_threshold {
            self.reports
                .append_persitent_warn(self.cycles, ReportContent::WarnHighTemp);
        } else {
            self.reports.remove_persitent_warn(&ReportContent::WarnHighTemp);
        }

        if self.stat.average_air_temp < params.monitoring.warn_low_temp_threshold {
            self.reports
                .append_persitent_warn(self.cycles, ReportContent::WarnLowTemp);
        } else {
            self.reports.remove_persitent_warn(&ReportContent::WarnLowTemp);
        }

        // Atmosphere warnings
        if self.atmo.partial_pressure(GasKind::Oxygen) < params.monitoring.warn_low_oxygen_threshold
        {
            self.reports
                .append_persitent_warn(self.cycles, ReportContent::WarnLowOxygen);
        } else {
            self.reports
                .remove_persitent_warn(&ReportContent::WarnLowOxygen);
        }

        if self.atmo.partial_pressure(GasKind::CarbonDioxide)
            < params.monitoring.warn_low_carbon_dioxide_threshold
        {
            self.reports
                .append_persitent_warn(self.cycles, ReportContent::WarnLowCarbonDioxide);
        } else {
            self.reports
                .remove_persitent_warn(&ReportContent::WarnLowCarbonDioxide);
        }
    }
}
