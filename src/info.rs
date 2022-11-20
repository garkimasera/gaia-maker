use crate::planet::Structure;

pub fn structure_info(structure: &Structure) -> String {
    match structure {
        Structure::None | Structure::Occupied { .. } => unreachable!(),
        _ => {
            t!(structure.kind().as_ref())
        }
    }
}
