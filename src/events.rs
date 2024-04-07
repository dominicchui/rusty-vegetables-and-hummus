use crate::ecology::{Cell, CellIndex, Ecosystem};

trait Event {
    fn apply_event_and_propagate(
        &self,
        ecosystem: &mut Ecosystem,
        cell_at: CellIndex,
    ) -> Option<CellIndex>;
}

enum Events {
    Rainfall,
    ThermalStress,
    Lightning,
    RockSlide,
    SandSlide,
    HumusSlide,
    Fire,
    Vegetation,
}

impl Event for Events {
    fn apply_event_and_propagate(
        &self,
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<CellIndex> {
        match self {
            Events::Rainfall => todo!(),
            Events::ThermalStress => todo!(),
            Events::Lightning => self.apply_and_propagate_lightning_event(ecosystem, index),
            Events::RockSlide => todo!(),
            Events::SandSlide => todo!(),
            Events::HumusSlide => todo!(),
            Events::Fire => todo!(),
            Events::Vegetation => todo!(),
        }
    }
}

impl Events {
    fn apply_and_propagate_lightning_event(
        &self,
        ecosystem: &mut Ecosystem,
        index: CellIndex,
    ) -> Option<CellIndex> {
        let cell = &mut ecosystem[index];

        // kill all vegetation at cell
        Self::kill_trees(cell);

        // destroy some bedrock and scatter as rocks and sand to nearby cells

        // does not propagate
        None
    }
}

impl Events {
    // converts all trees in a cell into dead vegetation
    fn kill_trees(cell: &mut Cell) {
        if let Some(trees) = cell.get_trees_layer_mut() {
            let biovolume = trees.get_tree_biomass();
            trees.number_of_plants = 0;
            trees.plant_height_sum = 0.0;
            trees.plant_age_sum = 0.0;
            cell.add_dead_vegetation(biovolume);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ecology::{Cell, CellLayer, Trees}, events::Events};
    #[test]
    fn kill_trees() {
        let layer = CellLayer::Trees(Trees {
            number_of_plants: 1,
            plant_height_sum: 30.0,
            plant_age_sum: 10.0
        });
        let mut cell = Cell {
            layers: vec![layer],
            soil_moisture: 0.0,
            sunlight: 0.0,
            temperature: 0.0,
        };
        let biomass = cell.get_tree_biomass();

        Events::kill_trees(&mut cell);
        
        let trees = cell.get_trees_layer();
        assert!(trees.is_some());
        let trees = trees.unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        let dead_vegetation = cell.get_dead_vegetation_layer();
        assert!(dead_vegetation.is_some());
        let dead_vegetation = dead_vegetation.unwrap();
        assert!(dead_vegetation.biomass == biomass);

        // add more trees and kill them
        let trees = cell.get_trees_layer_mut().unwrap();
        trees.number_of_plants = 5;
        trees.plant_height_sum = 150.0;
        let biomass_2 = cell.get_tree_biomass();

        Events::kill_trees(&mut cell);

        let trees = cell.get_trees_layer();
        assert!(trees.is_some());
        let trees = trees.unwrap();
        assert!(trees.number_of_plants == 0);
        assert!(trees.plant_age_sum == 0.0);
        assert!(trees.plant_height_sum == 0.0);

        let dead_vegetation = cell.get_dead_vegetation_layer();
        assert!(dead_vegetation.is_some());
        let dead_vegetation = dead_vegetation.unwrap();
        assert!(dead_vegetation.biomass == biomass + biomass_2);
    }
}