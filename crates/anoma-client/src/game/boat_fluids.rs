//! `Entity.updateFluidInteraction` for boats (from `VehicleEntity.tick`'s
//! `Entity.baseTick`): the currents of the water (and lava) the hull is in
//! push it, as they push items and TNT.
use super::boats::{Boat, HEIGHT};
use anoma_client::{
    motion::{FluidResources, FluidTags, LAVA, WATER},
    world::World,
};
use anoma_core::{
    geometry::{Aabb, Direction, Vec3},
    position::BlockPos,
};
use anoma_world::{
    entity_fluid::{
        CurrentTarget, EntityFluidInteraction, FluidInteractionWorld, FluidMembership,
        FluidSections, FluidTagId,
    },
    fluid::{FluidHeightWorld, FluidKind, FluidState},
    fluid_flow::{FluidFlowWorld, flow_at},
    registry::BlockStateId,
};
use std::convert::Infallible;

/// Water pushes at 0.014 a tick (`Entity.updateFluidInteraction`).
const WATER_PUSH: f64 = 0.014;

/// `Entity.updateFluidInteraction` for boat `b`: `isPushedByFluid`, so the
/// flow of the fluids its box (less 0.001) reaches adds to its motion.
pub(super) fn push_boat(b: &mut Boat, world: &World, fluids: &FluidResources) {
    let mut interaction = EntityFluidInteraction::new([WATER, LAVA]);
    let mut scene = BoatFluidWorld {
        boat: b,
        world,
        fluids,
    };
    if interaction.update(&mut scene, false, 4096).is_err() {
        return;
    }
    let lava_push = if world.fast_lava {
        0.007
    } else {
        0.0023333333333333335
    };
    let _ = interaction.apply_current(&mut scene, WATER, WATER_PUSH);
    let _ = interaction.apply_current(&mut scene, LAVA, lava_push);
}

struct BoatFluidWorld<'a> {
    boat: &'a mut Boat,
    world: &'a World,
    fluids: &'a FluidResources,
}
impl BoatFluidWorld<'_> {
    fn state(&self, p: BlockPos) -> BlockStateId {
        self.world.block(p.x, p.y, p.z)
    }
}
impl FluidMembership for BoatFluidWorld<'_> {
    fn fluid_in_tag(&self, kind: FluidKind, tag: FluidTagId) -> bool {
        FluidTags.fluid_in_tag(kind, tag)
    }
}
impl CurrentTarget for BoatFluidWorld<'_> {
    type Error = Infallible;
    fn is_player_class(&self) -> bool {
        false
    }
    fn velocity(&mut self) -> Result<Vec3, Infallible> {
        Ok(self.boat.velocity)
    }
    fn add_delta_movement(&mut self, delta: Vec3) -> Result<(), Infallible> {
        let v = self.boat.velocity;
        let sum = Vec3::new(v.x + delta.x, v.y + delta.y, v.z + delta.z);
        if delta.is_finite() && sum.is_finite() {
            self.boat.velocity = sum;
        }
        Ok(())
    }
}
impl FluidInteractionWorld for BoatFluidWorld<'_> {
    type Error = Infallible;
    type Chunk = ();
    fn interaction_box(&mut self) -> Result<Option<Aabb>, Infallible> {
        Ok(Some(
            self.boat.bounds().deflate(Vec3::new(0.001, 0.001, 0.001)),
        ))
    }
    fn loaded_chunk(&mut self, x: i32, z: i32) -> Result<Option<()>, Infallible> {
        Ok(self.world.chunks.contains_key(&(x, z)).then_some(()))
    }
    fn chunk_sections(&mut self, _: ()) -> Result<FluidSections<'_>, Infallible> {
        static SECTIONS: [bool; anoma_client::chunk::SECTIONS] =
            [true; anoma_client::chunk::SECTIONS];
        Ok(FluidSections {
            minimum_section_y: anoma_client::chunk::MIN_Y >> 4,
            has_fluid: &SECTIONS,
        })
    }
    fn bounding_box_min_y(&self) -> f64 {
        self.boat.position.y
    }
    fn block_x(&self) -> i32 {
        self.boat.position.x.floor() as i32
    }
    fn eye_y(&mut self) -> Result<f64, Infallible> {
        Ok(self.boat.position.y + HEIGHT)
    }
    fn block_z(&self) -> i32 {
        self.boat.position.z.floor() as i32
    }
    fn fluid_state(&mut self, p: BlockPos) -> Result<FluidState, Infallible> {
        Ok(self.fluids.fluid(self.state(p)))
    }
    fn fluid_height(&mut self, f: FluidState, p: BlockPos) -> Result<f32, Infallible> {
        f.height_at(self, [p.x, p.y, p.z])
    }
    fn camera_height(&mut self, f: FluidState, p: BlockPos) -> Result<f32, Infallible> {
        f.camera_height_at(self, [p.x, p.y, p.z])
    }
    fn flow(&mut self, f: FluidState, p: BlockPos) -> Result<Vec3, Infallible> {
        flow_at(self, f, p)
    }
}
impl FluidHeightWorld for BoatFluidWorld<'_> {
    type Error = Infallible;
    fn fluid(&mut self, p: [i32; 3]) -> Result<FluidState, Infallible> {
        Ok(self.fluids.fluid(self.world.block(p[0], p[1], p[2])))
    }
    fn sturdy_down(&mut self, p: [i32; 3]) -> Result<bool, Infallible> {
        let state = self.world.block(p[0], p[1], p[2]);
        Ok(self.fluids.sturdy(state, Direction::Down))
    }
}
impl FluidFlowWorld for BoatFluidWorld<'_> {
    type Error = Infallible;
    type Block = BlockStateId;
    fn fluid(&mut self, p: BlockPos) -> Result<FluidState, Infallible> {
        Ok(self.fluids.fluid(self.state(p)))
    }
    fn block(&mut self, p: BlockPos) -> Result<BlockStateId, Infallible> {
        Ok(self.state(p))
    }
    fn blocks_fluid_flow(&self, s: BlockStateId) -> bool {
        self.fluids.blocks_flow(s)
    }
    fn is_ice_class(&self, s: BlockStateId) -> bool {
        self.fluids.ice(s)
    }
    fn is_face_sturdy(
        &mut self,
        s: BlockStateId,
        _: BlockPos,
        d: Direction,
    ) -> Result<bool, Infallible> {
        Ok(self.fluids.sturdy(s, d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A wave: water flowing off a one-block step falls into the water
    /// below. A boat driven under it is pushed back out, away from the step
    /// and down, the way the falling water flows.
    #[test]
    fn falling_water_off_a_step_pushes_a_boat_back_out() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/anoma-client")
            .join(anoma_core::VANILLA_VERSION);
        if !root.join("movement-properties.nbt").exists() {
            return;
        }
        let assets = anoma_client::assets::Assets::load(&root).unwrap();
        let render = assets.reload(&root).unwrap();
        let facts = anoma_client::blocks::BlockFacts::load(&root, assets.registry).unwrap();
        let fluids = FluidResources::new(&assets, &facts).unwrap();
        let mut world = anoma_client::world::World::test_scene(&assets, &render, 3).unwrap();
        let stone = assets.registry.default_state("minecraft:stone").unwrap();
        let water = assets.registry.default_state("minecraft:water").unwrap();
        let with_level = |level: &str| {
            let mut state = assets.registry.state(water).unwrap().clone();
            state.properties.insert("level".into(), level.into());
            assets.registry.resolve(&state).unwrap()
        };
        // Lower pool at y 4 (x 21..=28), the step at x 20 with its water at
        // y 5, flowing over the edge (x 21, y 5) and falling into x 21, y 4.
        for x in 17..=28 {
            for z in 17..=23 {
                world.set(x, 3, z, stone).unwrap();
                if x <= 20 {
                    world.set(x, 4, z, stone).unwrap();
                    world.set(x, 5, z, water).unwrap();
                } else {
                    world.set(x, 4, z, water).unwrap();
                }
            }
        }
        for z in 17..=23 {
            world.set(21, 5, z, with_level("1")).unwrap();
            world.set(21, 4, z, with_level("8")).unwrap();
        }
        // Floating on the pool against the step, its bow under the wave.
        let mut boat = Boat::new(1, Arc::from("oak"), Vec3::new(21.7, 4.45, 20.5), 90.0);
        push_boat(&mut boat, &world, &fluids);
        assert!(
            boat.velocity.x > 0.0,
            "pushed away from the step: {:?}",
            boat.velocity
        );
        assert!(
            boat.velocity.y < 0.0,
            "and down, with the falling water: {:?}",
            boat.velocity
        );
        let push = boat.velocity.length();
        assert!((push - WATER_PUSH).abs() < 1e-9, "push {push}");
        // Still water has no current.
        let mut calm = Boat::new(2, Arc::from("oak"), Vec3::new(24.5, 4.45, 20.5), 0.0);
        for z in 17..=23 {
            world.set(21, 5, z, stone).unwrap();
            world.set(21, 4, z, water).unwrap();
        }
        push_boat(&mut calm, &world, &fluids);
        assert_eq!(calm.velocity, Vec3::ZERO);
    }
}
