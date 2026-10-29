//! Running a model's animation for one entity in one frame: the frame's
//! shared values and each entity's `var.` values, kept from frame to frame.
use super::{
    model::CemModel,
    program::{self, CHANNELS, INPUTS, VISIBLE, VISIBLE_BOXES},
};
use crate::mobs::anim::Poses;
use std::{cell::RefCell, collections::HashMap};

/// What every entity reads alike this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInputs {
    pub player_pos: [f32; 3],
    /// Pitch and yaw, radians.
    pub player_rot: [f32; 2],
    /// World game time and day time in ticks, partial tick included.
    pub game_time: f64,
    pub day_time: f64,
    pub paused: bool,
}

#[derive(Default)]
struct Entity {
    vars: Vec<f32>,
    seen: u64,
}

#[derive(Default)]
struct Runtime {
    frame: FrameInputs,
    /// Seconds since the last frame (0.25 at most: a hitch is no long step).
    frame_time: f32,
    last: Option<std::time::Instant>,
    counter: u64,
    entities: HashMap<u32, Entity>,
    regs: Vec<f32>,
}

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::default());
}

/// Starts a frame: the counter moves on and entities not drawn for a while
/// forget their variables.
pub fn begin_frame(frame: FrameInputs) {
    RUNTIME.with(|r| {
        let mut r = r.borrow_mut();
        let now = std::time::Instant::now();
        r.frame_time = r
            .last
            .map_or(0.05, |l| now.duration_since(l).as_secs_f32().min(0.25));
        r.last = Some(now);
        r.frame = frame;
        r.counter += 1;
        if r.counter % 600 == 0 {
            let now = r.counter;
            r.entities.retain(|_, e| now - e.seen < 600);
        }
    });
}

/// The input registers the frame fills in.
fn frame_values(r: &Runtime, regs: &mut [f32]) {
    let f = &r.frame;
    let mut set = |name: &str, v: f32| {
        if let Some(i) = program::input(name) {
            regs[i] = v;
        }
    };
    const WRAP: f64 = 27720.0;
    set("frame_time", if f.paused { 1e-9 } else { r.frame_time });
    set("frame_counter", (r.counter % 27720) as f32);
    set("player_pos_x", f.player_pos[0]);
    set("player_pos_y", f.player_pos[1]);
    set("player_pos_z", f.player_pos[2]);
    set("player_rot_x", f.player_rot[0]);
    set("player_rot_y", f.player_rot[1]);
    set("time", (f.game_time % WRAP) as f32);
    set("day_time", (f.day_time % 24000.0) as f32);
    set("day_count", (f.day_time / 24000.0).floor() as f32);
    set("is_paused", if f.paused { 1.0 } else { 0.0 });
}

/// Poses `model` for entity `id` over the vanilla pose already in `poses`:
/// `inputs` holds the entity's values in [`INPUTS`] order.
pub fn animate(id: u32, model: &CemModel, inputs: &[f32], poses: &mut Poses) {
    let p = &model.program;
    if p.steps.is_empty() {
        return;
    }
    RUNTIME.with(|r| {
        let mut r = r.borrow_mut();
        let r = &mut *r;
        let mut regs = std::mem::take(&mut r.regs);
        regs.clear();
        regs.resize(p.registers(), 0.0);
        regs[..INPUTS.len()].copy_from_slice(&inputs[..INPUTS.len()]);
        frame_values(r, &mut regs);
        let counter = r.counter;
        // Distance to the player, in blocks.
        if let (Some(d), Some(x)) = (program::input("distance"), program::input("pos_x")) {
            let p = r.frame.player_pos;
            regs[d] = ((regs[x] - p[0]).powi(2) + (regs[x + 1] - p[1]).powi(2) + (regs[x + 2] - p[2]).powi(2)).sqrt();
        }
        let entity = r.entities.entry(id).or_default();
        entity.seen = counter;
        let start = p.locals_start();
        for (i, &slot) in p.local_ids.iter().enumerate() {
            regs[start + i] = entity.vars.get(slot as usize).copied().unwrap_or(0.0);
        }
        for (i, name) in p.parts.iter().enumerate() {
            let pose = *poses.get(name);
            let base = p.channel(i, 0);
            regs[base..base + 9].copy_from_slice(&pose);
            regs[base + VISIBLE] = if poses.hidden_named(name) { 0.0 } else { 1.0 };
            regs[base + VISIBLE_BOXES] = if poses.skips_named(name) { 0.0 } else { 1.0 };
        }
        p.run(&mut regs);
        // ANOMA_CEM_TRACE=<entity id>: that entity's inputs and variables
        // every 20 frames, in the log.
        // ANOMA_CEM_TRACE=all: every entity, every 200 frames.
        static TRACE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
        let trace = TRACE.get_or_init(|| std::env::var("ANOMA_CEM_TRACE").ok());
        let every = if trace.as_deref() == Some("all") { 200 } else { 20 };
        if trace
            .as_deref()
            .is_some_and(|t| t == "all" || t.parse() == Ok(id))
            && counter % every == 0
        {
            let show = |n: &str| program::input(n).map_or(f32::NAN, |i| regs[i]);
            let vars: Vec<String> = p
                .locals
                .iter()
                .enumerate()
                .take(if every == 200 { 0 } else { 24 })
                .map(|(i, n)| format!("{n}={:.3}", regs[start + i]))
                .collect();
            log::info!(
                "cem {id} {:.1},{:.1},{:.1}: ground={} water={} limb_swing={:.3} limb_speed={:.3} frame_time={:.4} age={:.1} | {}",
                show("pos_x"),
                show("pos_y"),
                show("pos_z"),
                show("is_on_ground"),
                show("is_in_water"),
                show("limb_swing"),
                show("limb_speed"),
                show("frame_time"),
                show("age"),
                vars.join(" ")
            );
        }
        for (i, &slot) in p.local_ids.iter().enumerate() {
            let slot = slot as usize;
            if entity.vars.len() <= slot {
                entity.vars.resize(slot + 1, 0.0);
            }
            entity.vars[slot] = regs[start + i];
        }
        for (i, name) in p.parts.iter().enumerate() {
            let base = p.channel(i, 0);
            let pose = poses.get(name);
            for (c, v) in pose.iter_mut().enumerate() {
                if regs[base + c].is_finite() {
                    *v = regs[base + c];
                }
            }
            poses.visible(name, regs[base + VISIBLE] != 0.0);
            poses.set_skip_draw(name, regs[base + VISIBLE_BOXES] == 0.0);
        }
        debug_assert_eq!(CHANNELS.len(), 11);
        r.regs = regs;
    });
}
