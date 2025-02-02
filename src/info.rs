use crate::planet::Structure;

pub fn structure_info(structure: &Structure) -> String {
    match structure {
        Structure::Settlement(settlement) => {
            let s = if settlement.pop >= 10.0 {
                t!("city")
            } else {
                t!("settlement")
            };
            format!(
                "{} ({}, {})",
                s,
                t!("animal", settlement.id),
                t!("age", settlement.age),
            )
        }
        _ => {
            t!(structure.kind().as_ref())
        }
    }
}
