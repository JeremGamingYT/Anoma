//! A dead or dying mob neither thinks nor acts: `LivingEntity.aiStep` skips
//! `serverAiStep` while `isImmobile` (`isDeadOrDying`), and
//! `LivingEntity.heal` never raises a body in its death animation.
use super::{
    Mob, MobData, MobEvent, MobType, MobTypes, MobWorld, PlayerView, Species, ai,
    spawning::LocalDifficulty,
};
use crate::{
    assets::Assets,
    biome::Biomes,
    blocks::BlockFacts,
    chunk::{Chunk, MIN_Y},
    collisions,
    items::Items,
    motion::MotionResources,
    world::World,
};
use anoma_core::{
    geometry::{Aabb, Vec3},
    random::LegacyRandom,
};
use anoma_world::registry::BlockRegistry;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// The test cow, made `species` with `max_health`.
fn kind(id: &str, species: Species, max_health: f32) -> Arc<MobType> {
    let mut k = (*super::sensor_scan_tests::kind()).clone();
    k.id = id.into();
    k.species = species;
    k.max_health = max_health;
    Arc::new(k)
}

/// `/kill`: `Entity.kill` hurts with `genericKill` (it bypasses
/// invulnerability), the health drops to 0 and the death animation starts.
fn kill(m: &mut Mob) {
    m.health = 0.0;
    m.death_time = m.death_time.max(1);
}

#[test]
fn a_killed_mob_is_immobile() {
    let mut m = Mob::new(
        1,
        kind("minecraft:zombie", Species::Zombie, 20.0),
        Vec3::ZERO,
        0.0,
    );
    assert!(!ai::is_immobile(&m), "a living zombie thinks");
    kill(&mut m);
    assert!(ai::is_immobile(&m), "a killed zombie gets no serverAiStep");
}

/// `WitherSkull.onHitEntity` heals its owner 5 when the skull kills: a
/// wither killed while its skull flew must stay dead, not stand up in its
/// death pose and fire again.
#[test]
fn healing_never_revives_a_dying_mob() {
    let mut w = Mob::new(
        1,
        kind("minecraft:wither", Species::Wither, 300.0),
        Vec3::ZERO,
        0.0,
    );
    w.health = 298.0;
    w.heal(5.0);
    assert_eq!(
        w.health, 300.0,
        "a living wither heals up to its max health"
    );
    kill(&mut w);
    w.heal(5.0);
    assert_eq!(w.health, 0.0);
    assert!(!w.alive() && ai::is_immobile(&w));
}

/// The prepared resources (`ANOMA_TEST_RESOURCES`, else `.local`).
fn resources() -> Option<PathBuf> {
    let root = std::env::var_os("ANOMA_TEST_RESOURCES")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.local/anoma-client")
                .join(anoma_core::VANILLA_VERSION)
        });
    root.join("anoma-resources.json").exists().then_some(root)
}

/// Five by five chunks of stone floor at `MIN_Y`, air above.
fn flat_world(registry: &BlockRegistry) -> World {
    let state = |name: &str| registry.default_state(name).expect(name);
    let (air, stone) = (state("minecraft:air"), state("minecraft:stone"));
    let mut world = World::new(air, 0);
    for cz in -2..=2 {
        for cx in -2..=2 {
            world
                .chunks
                .insert((cx, cz), Arc::new(Chunk::empty(cx, cz)));
            for z in 0..16 {
                for x in 0..16 {
                    world
                        .set(cx * 16 + x, MIN_Y, cz * 16 + z, stone)
                        .expect("floor");
                }
            }
        }
    }
    world.game_time = 1000;
    // Night: no undead burning in the sun.
    world.day_time = 18_000;
    world
}

/// A survival player standing at `at`.
fn player(at: Vec3) -> PlayerView {
    PlayerView {
        position: at,
        bounds: Aabb::new(
            Vec3::new(at.x - 0.3, at.y, at.z - 0.3),
            Vec3::new(at.x + 0.3, at.y + 1.8, at.z + 0.3),
        ),
        eye: [at.x, at.y + f64::from(1.62_f32), at.z],
        alive: true,
        creative: false,
        sneaking: false,
        swimming: false,
        look: [0.0, 0.0, 1.0],
        main_hand: None,
        off_hand: None,
        head: None,
        armor_cover: 0.0,
        invisible: false,
        spectator: false,
        gold_armor: false,
        hurt_by: None,
        last_hurt: None,
        velocity: Vec3::ZERO,
        yaw: 0.0,
        sprinting: false,
        riding: false,
        hero: false,
        effects: Vec::new(),
        uuid: [1, 2, 3, 4],
        allies: Default::default(),
        remote: false,
        sleeping: false,
        sleep_timer: 0,
        passenger: false,
    }
}

/// An attack of any kind: melee, a sting, a projectile (arrow, skull...).
fn attacks(e: &MobEvent) -> bool {
    matches!(
        e,
        MobEvent::AttackPlayer(..)
            | MobEvent::AttackPlayerWithEffect { .. }
            | MobEvent::AttackMob { .. }
            | MobEvent::DamagePlayer { .. }
            | MobEvent::Projectile(_)
            | MobEvent::SonicBoom { .. }
    )
}

/// What a mob did after `/kill` caught it mid-attack.
struct Dying {
    /// Its ticks until its body went (`tickDeath`).
    ticks: i32,
    attacked: bool,
    /// Movement input (`xxa`, `zza`, jumping) on any of its dying ticks.
    steered: bool,
    /// Its health once gone: never back above 0.
    health: f32,
}

impl Dying {
    /// One death tick's record. On the third, a skull still in flight kills
    /// something (`WitherSkull.onHitEntity`: `owner.heal(5.0F)`).
    fn record(&mut self, m: &mut Mob, events: &[MobEvent]) {
        self.ticks += 1;
        if self.ticks == 3 {
            m.heal(5.0);
        }
        self.attacked |= events.iter().any(attacks);
        self.steered |= m.ai.xxa != 0.0 || m.ai.zza != 0.0 || m.ai.jumping;
        self.health = self.health.max(m.health);
    }
}

/// What a `MobWorld` borrows, loaded from the prepared resources.
struct Res {
    assets: Assets,
    facts: BlockFacts,
    motion: MotionResources,
    data: MobData,
    types: MobTypes,
    biomes: Biomes,
    items: Items,
}
impl Res {
    fn load() -> Option<Self> {
        let root = resources()?;
        let assets = Assets::load(&root).expect("assets");
        let facts = BlockFacts::load(&root, assets.registry).expect("block facts");
        Some(Self {
            motion: MotionResources::load(&root, &assets).expect("motion"),
            data: MobData::load(&root, &facts),
            types: MobTypes::load(&root).expect("mob types"),
            biomes: Biomes::load(&root).expect("biomes"),
            items: Items::load(&root, assets.registry, &facts).expect("items"),
            assets,
            facts,
        })
    }
}

/// Everything a mob tick reads, on the flat world at night on Normal.
struct Scene<'a> {
    res: &'a Res,
    world: World,
    collision: anoma_world::block_collision::BlockCollisionRegistry<'a>,
    lid_openers: HashMap<(i32, i32, i32), i32>,
    player: PlayerView,
}
impl<'a> Scene<'a> {
    fn new(res: &'a Res, player_at: Vec3) -> Self {
        let registry = res.assets.registry;
        let root = resources().expect("resources");
        Self {
            res,
            world: flat_world(registry),
            collision: collisions::load(&root, registry).expect("collisions"),
            lid_openers: HashMap::new(),
            player: player(player_at),
        }
    }
    fn mob_world<'s>(&'s self, others: &'s [super::Seen]) -> MobWorld<'s> {
        let (res, world) = (self.res, &self.world);
        MobWorld {
            teams: Default::default(),
            world,
            registry: res.assets.registry,
            facts: &res.facts,
            fluid_tables: &res.motion.fluids,
            movement: &res.motion.blocks,
            collision: &self.collision,
            data: &res.data,
            player: Some(self.player.clone()),
            nearest_player: Some(self.player.position),
            others,
            others_grid: None,
            others_index: None,
            daylight: false,
            game_time: world.game_time,
            day_time: world.day_time,
            raining: false,
            thundering: false,
            nether: false,
            block_tags: &res.items.block_tags,
            biomes: &res.biomes,
            mob_griefing: false,
            mob_drops: true,
            difficulty: LocalDifficulty::new(2, world.day_time, 0, 0.0),
            end: false,
            world_seed: 0,
            sea_level: -63,
            items: Some(&res.items),
            lid_openers: &self.lid_openers,
        }
    }
    /// `ai::tick` once (as `ServerLevel.tick` runs it); false once gone.
    fn tick(&mut self, m: &mut Mob, random: &mut LegacyRandom, events: &mut Vec<MobEvent>) -> bool {
        self.world.game_time += 1;
        let snapshot = vec![m.seen(Some(self.player.uuid))];
        ai::tick(m, &self.mob_world(&snapshot), random, events)
    }
}

/// `kind` next to a survival player at night: ticked until its first
/// attack, killed as `/kill` kills, then ticked until its body is removed.
/// None without the prepared resources.
fn kill_mid_attack(kind: &str, at: Vec3, player_at: Vec3) -> Option<Dying> {
    let res = Res::load()?;
    let mut scene = Scene::new(&res, player_at);
    let t = Arc::new(res.types.types.get(kind).expect(kind).clone());
    let mut m = Mob::new(1, t, at, 0.0);
    m.persistent = true;
    let mut random = LegacyRandom::new(0);
    let mut killed: Option<Dying> = None;
    for _ in 0..400 {
        let mut events = Vec::new();
        let kept = scene.tick(&mut m, &mut random, &mut events);
        match killed.as_mut() {
            Some(d) => {
                d.record(&mut m, &events);
                if !kept {
                    break;
                }
            }
            None if events.iter().any(attacks) => {
                kill(&mut m);
                killed = Some(Dying {
                    ticks: 0,
                    attacked: false,
                    steered: false,
                    health: 0.0,
                });
            }
            None => {}
        }
    }
    Some(killed.expect("the mob attacked the player before it was killed"))
}

fn assert_dies_still(d: Dying, what: &str) {
    assert!(!d.attacked, "a killed {what} attacked while dying");
    assert!(!d.steered, "a killed {what} had movement input while dying");
    assert_eq!(d.health, 0.0, "a killed {what} came back to life");
    // LivingEntity.tickDeath: removed on the twentieth death tick (`/kill`
    // counts the first).
    assert_eq!(
        d.ticks, 19,
        "a killed {what}'s body goes after its death animation"
    );
}

#[test]
fn a_zombie_killed_mid_attack_neither_attacks_nor_moves() {
    let floor = f64::from(MIN_Y + 1);
    let Some(d) = kill_mid_attack(
        "minecraft:zombie",
        Vec3::new(0.5, floor, 0.5),
        Vec3::new(1.5, floor, 0.5),
    ) else {
        return;
    };
    assert_dies_still(d, "zombie");
}

#[test]
fn a_wither_killed_mid_volley_fires_no_skull() {
    let floor = f64::from(MIN_Y + 1);
    let Some(d) = kill_mid_attack(
        "minecraft:wither",
        Vec3::new(0.5, floor + 4.0, 0.5),
        Vec3::new(8.5, floor, 0.5),
    ) else {
        return;
    };
    assert_dies_still(d, "wither");
}
