use super::*;

impl Planet {
    pub fn edit_biome(&mut self, coords: Coords, biome: Biome) {
        self.map[coords].biome = biome;
    }
}
