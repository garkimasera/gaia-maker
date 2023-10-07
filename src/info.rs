use crate::planet::Structure;

pub fn structure_info(structure: &Structure) -> String {
    match structure {
        Structure::None | Structure::Occupied { .. } => unreachable!(),
        Structure::Settlement { settlement } => {
            format!("{} ({})", t!("settlement"), t!(settlement.age.as_ref()))
        }
        _ => {
            t!(structure.kind().as_ref())
        }
    }
}
