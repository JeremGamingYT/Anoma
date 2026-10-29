//! A model's animation lines compiled against one register file: the entity
//! values first, then the `var.`/`varb.` values the entity keeps from frame to
//! frame, then each part's channels. A line stores its value at once, so the
//! lines after it read the new value, as both mods do.
use super::expr::{self, Node};

/// The values read from the entity and the world, in register order.
pub const INPUTS: &[&str] = &[
    "limb_swing",
    "limb_speed",
    "age",
    "head_yaw",
    "head_pitch",
    "frame_time",
    "frame_counter",
    "swing_progress",
    "hurt_time",
    "death_time",
    "health",
    "max_health",
    "id",
    "pos_x",
    "pos_y",
    "pos_z",
    "rot_x",
    "rot_y",
    "player_pos_x",
    "player_pos_y",
    "player_pos_z",
    "player_rot_x",
    "player_rot_y",
    "time",
    "day_time",
    "day_count",
    "rule_index",
    "move_forward",
    "move_strafing",
    "fluid_depth",
    "fluid_depth_up",
    "fluid_depth_down",
    "distance",
    "anger_time",
    "anger_time_start",
    "is_alive",
    "is_on_ground",
    "is_in_water",
    "is_in_lava",
    "is_hurt",
    "is_riding",
    "is_ridden",
    "is_child",
    "is_sneaking",
    "is_sprinting",
    "is_aggressive",
    "is_burning",
    "is_wet",
    "is_tamed",
    "is_sitting",
    "is_invisible",
    "is_glowing",
    "is_jumping",
    "is_swimming",
    "is_gliding",
    "is_climbing",
    "is_crawling",
    "is_blocking",
    "is_using_item",
    "is_right_handed",
    "is_swinging_right_arm",
    "is_swinging_left_arm",
    "is_holding_item_right",
    "is_holding_item_left",
    "is_on_shoulder",
    "is_on_head",
    "is_first_person_hand",
    "is_in_hand",
    "is_in_gui",
    "is_in_item_frame",
    "is_paused",
];
/// Index of an input in [`INPUTS`] (and in the register file).
pub fn input(name: &str) -> Option<usize> {
    INPUTS.iter().position(|n| *n == name)
}

/// A part's animated values, in register order within the part.
pub const CHANNELS: [&str; 11] = [
    "tx",
    "ty",
    "tz",
    "rx",
    "ry",
    "rz",
    "sx",
    "sy",
    "sz",
    "visible",
    "visible_boxes",
];
pub const VISIBLE: usize = 9;
pub const VISIBLE_BOXES: usize = 10;

/// The slot of a `var.`/`varb.` name in every entity's variables.
pub fn intern(name: &str) -> u32 {
    static NAMES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    let mut names = NAMES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match names.iter().position(|n| n == name) {
        Some(i) => i as u32,
        None => {
            names.push(name.to_owned());
            (names.len() - 1) as u32
        }
    }
}

/// One line: the register it writes and the value.
#[derive(Clone, Debug)]
pub struct Step {
    pub target: u16,
    pub value: Node,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub steps: Vec<Step>,
    /// `var.` and `varb.` names, by register after the inputs.
    pub locals: Vec<String>,
    /// Each local's slot in the entity's variables, which every model drawn
    /// for the entity shares (its body, armour and layers).
    pub local_ids: Vec<u32>,
    /// The parts a line reads or writes, by their name in the built model,
    /// with their channels after the locals.
    pub parts: Vec<String>,
    /// Lines that did not compile: (target, error), reported once at load.
    pub errors: Vec<(String, String)>,
}

impl Program {
    pub fn locals_start(&self) -> usize {
        INPUTS.len() + 1
    }
    /// The register that always reads 0: names this game doesn't know read
    /// 0 (false for an `is_` flag), as in EMF.
    pub fn zero(&self) -> usize {
        INPUTS.len()
    }
    pub fn part_start(&self) -> usize {
        self.locals_start() + self.locals.len()
    }
    pub fn registers(&self) -> usize {
        self.part_start() + self.parts.len() * CHANNELS.len()
    }
    pub fn channel(&self, part: usize, channel: usize) -> usize {
        self.part_start() + part * CHANNELS.len() + channel
    }

    /// Compiles `lines` in order; `part` gives the model part a name in a
    /// line stands for (`head` the vanilla head, `head2` a custom part).
    pub fn compile(lines: &[(String, String)], part: &dyn Fn(&str) -> Option<String>) -> Self {
        // Every name gets its register before any line compiles, so a line
        // may read a variable a later line writes (its last frame's value).
        let mut locals: Vec<String> = Vec::new();
        let mut parts: Vec<String> = Vec::new();
        fn place(
            name: &str,
            locals: &mut Vec<String>,
            parts: &mut Vec<String>,
            resolve: &dyn Fn(&str) -> Option<String>,
        ) -> bool {
            if name.starts_with("var.") || name.starts_with("varb.") {
                if !locals.iter().any(|l| l == name) {
                    locals.push(name.to_owned());
                }
                return true;
            }
            if let Some((part, channel)) = name.rsplit_once('.')
                && CHANNELS.contains(&channel)
                && let Some(part) = resolve(part)
            {
                if !parts.contains(&part) {
                    parts.push(part);
                }
                return true;
            }
            false
        }
        for (target, text) in lines {
            place(target, &mut locals, &mut parts, part);
            let _ = expr::compile(text, &mut |n| {
                place(n, &mut locals, &mut parts, part);
                Some(0)
            });
        }
        let local_ids = locals.iter().map(|l| intern(l)).collect();
        let mut program = Self {
            locals,
            local_ids,
            parts,
            ..Self::default()
        };
        for (target, text) in lines {
            // An unknown target is a line that does nothing.
            let Some(slot) = program.slot(target, part) else {
                continue;
            };
            let zero = program.zero();
            let compiled = expr::compile(text, &mut |n| {
                Some(program.slot(n, part).unwrap_or(zero) as u16)
            });
            match compiled {
                Ok(value) => program.steps.push(Step {
                    target: slot as u16,
                    value,
                }),
                Err(e) => program.errors.push((target.clone(), e)),
            }
        }
        program
    }

    /// The register a name reads or writes.
    fn slot(&self, name: &str, resolve: &dyn Fn(&str) -> Option<String>) -> Option<usize> {
        if let Some(i) = input(name) {
            return Some(i);
        }
        if let Some(i) = self.locals.iter().position(|l| l == name) {
            return Some(self.locals_start() + i);
        }
        if let Some((part, channel)) = name.rsplit_once('.')
            && let Some(c) = CHANNELS.iter().position(|x| *x == channel)
            && let Some(part) = resolve(part)
            && let Some(p) = self.parts.iter().position(|x| *x == part)
        {
            return Some(self.channel(p, c));
        }
        None
    }

    /// Runs every line; a NaN result leaves its target as it was.
    pub fn run(&self, regs: &mut [f32]) {
        for step in &self.steps {
            let v = step.value.eval(regs);
            if !v.is_nan() {
                regs[step.target as usize] = v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(l: &[(&str, &str)]) -> Vec<(String, String)> {
        l.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn lines_run_in_order_and_read_what_earlier_lines_wrote() {
        let p = Program::compile(
            &lines(&[
                ("var.a", "var.b + 1"),
                ("var.b", "var.a * 2"),
                ("head.rx", "var.b + head.rx + limb_speed"),
                ("varb.x", "is_unheard_of || true"),
            ]),
            &|p| (p == "head").then(|| p.to_owned()),
        );
        assert!(p.errors.is_empty(), "{:?}", p.errors);
        let mut regs = vec![0.0; p.registers()];
        regs[input("limb_speed").unwrap()] = 0.5;
        regs[p.channel(0, 3)] = 0.25;
        p.run(&mut regs);
        let b = regs[p.locals_start() + 1];
        assert_eq!(b, 2.0);
        assert_eq!(regs[p.channel(0, 3)], 2.75);
        // Next frame: var.a starts from last frame's var.b.
        p.run(&mut regs);
        assert_eq!(regs[p.locals_start()], 3.0);
        assert_eq!(regs[p.locals_start() + 2], 1.0);
    }

    #[test]
    fn unknown_targets_do_nothing_and_unknown_names_read_zero() {
        let p = Program::compile(
            &lines(&[
                ("tail.rx", "1"),
                ("var.a", "mystery + 1"),
                ("var.b", "sin("),
            ]),
            &|p| (p == "head").then(|| p.to_owned()),
        );
        assert_eq!(p.steps.len(), 1);
        assert_eq!(p.errors.len(), 1);
        let mut regs = vec![0.0; p.registers()];
        p.run(&mut regs);
        assert_eq!(regs[p.locals_start()], 1.0);
    }
}
