//! OptiFine's part names (`leg1`, `headwear`, `bill`...) for the vanilla model
//! parts they stand for, by model family. A name no table lists is the vanilla
//! part's own name.

const QUADRUPED: &[(&str, &str)] = &[
    ("leg1", "right_hind_leg"),
    ("leg2", "left_hind_leg"),
    ("leg3", "right_front_leg"),
    ("leg4", "left_front_leg"),
];
const BIPED: &[(&str, &str)] = &[("headwear", "hat")];
const VILLAGER: &[(&str, &str)] = &[
    ("headwear", "hat"),
    ("headwear2", "hat_rim"),
    ("bodywear", "jacket"),
];
const CHICKEN: &[(&str, &str)] = &[("bill", "beak"), ("chin", "red_thing")];
const SPIDER: &[(&str, &str)] = &[
    ("neck", "body0"),
    ("body", "body1"),
    ("leg1", "right_hind_leg"),
    ("leg2", "left_hind_leg"),
    ("leg3", "right_middle_hind_leg"),
    ("leg4", "left_middle_hind_leg"),
    ("leg5", "right_middle_front_leg"),
    ("leg6", "left_middle_front_leg"),
    ("leg7", "right_front_leg"),
    ("leg8", "left_front_leg"),
];
const WOLF: &[(&str, &str)] = &[
    ("mane", "upper_body"),
    ("leg1", "right_hind_leg"),
    ("leg2", "left_hind_leg"),
    ("leg3", "right_front_leg"),
    ("leg4", "left_front_leg"),
];
const CAT: &[(&str, &str)] = &[
    ("tail", "tail1"),
    ("back_left_leg", "left_hind_leg"),
    ("back_right_leg", "right_hind_leg"),
    ("front_left_leg", "left_front_leg"),
    ("front_right_leg", "right_front_leg"),
];
const HORSE: &[(&str, &str)] = &[
    ("mouth", "upper_mouth"),
    ("neck", "head_parts"),
    ("back_left_leg", "left_hind_leg"),
    ("back_right_leg", "right_hind_leg"),
    ("front_left_leg", "left_front_leg"),
    ("front_right_leg", "right_front_leg"),
    ("headpiece", "head_saddle"),
    ("noseband", "mouth_saddle_wrap"),
    ("right_rein", "right_saddle_line"),
    ("left_rein", "left_saddle_line"),
    ("right_bit", "right_saddle_mouth"),
    ("left_bit", "left_saddle_mouth"),
];
const BEE: &[(&str, &str)] = &[("body", "bone"), ("torso", "body")];
const SLIME: &[(&str, &str)] = &[("body", "cube")];
const SQUID: &[(&str, &str)] = &[
    ("tentacle1", "tentacle0"),
    ("tentacle2", "tentacle1"),
    ("tentacle3", "tentacle2"),
    ("tentacle4", "tentacle3"),
    ("tentacle5", "tentacle4"),
    ("tentacle6", "tentacle5"),
    ("tentacle7", "tentacle6"),
    ("tentacle8", "tentacle7"),
];
const HOGLIN: &[(&str, &str)] = &[
    ("back_left_leg", "left_hind_leg"),
    ("back_right_leg", "right_hind_leg"),
    ("front_left_leg", "left_front_leg"),
    ("front_right_leg", "right_front_leg"),
];
const LLAMA: &[(&str, &str)] = &[
    ("leg1", "right_hind_leg"),
    ("leg2", "left_hind_leg"),
    ("leg3", "right_front_leg"),
    ("leg4", "left_front_leg"),
    ("chest_left", "left_chest"),
    ("chest_right", "right_chest"),
];
const BABY_VILLAGER: &[(&str, &str)] = &[
    ("headwear", "hat"),
    ("headwear2", "hat_rim"),
    ("bodywear", "bb_main"),
];

/// The model a jem name belongs to, without its layer and baby suffixes.
fn base(jem: &str) -> &str {
    let mut s = jem;
    for suffix in [
        "_inner_armor",
        "_outer_armor",
        "_helmet",
        "_chestplate",
        "_leggings",
        "_boots",
        "_outer",
        "_collar",
        "_armor",
        "_saddle",
        "_decor",
        "_wool_undercoat",
        "_wool",
        "_charge",
        "_harness",
        "_ropes",
        "_baby",
    ] {
        s = s.strip_suffix(suffix).unwrap_or(s);
    }
    s
}

/// The OptiFine names of the model `jem` names (`cow`, `zombie_baby_boots`).
pub fn table(jem: &str) -> &'static [(&'static str, &'static str)] {
    let baby = jem.contains("_baby");
    match base(jem) {
        "cow" | "mooshroom" | "warm_cow" | "cold_cow" | "temperate_cow" | "pig" | "warm_pig"
        | "cold_pig" | "temperate_pig" | "sheep" | "panda" | "polar_bear" | "creeper" | "fox"
        | "goat" | "axolotl" | "turtle" | "sniffer" | "armadillo" => QUADRUPED,
        "zombie" | "husk" | "drowned" | "giant" | "skeleton" | "stray" | "wither_skeleton"
        | "bogged" | "parched" | "enderman" | "zombie_villager" | "player" | "player_slim"
        | "piglin" | "piglin_brute" | "zombified_piglin" | "evoker" | "illusioner" | "pillager"
        | "vindicator" | "witch" | "armor_stand" => BIPED,
        "villager" | "wandering_trader" if baby => BABY_VILLAGER,
        "villager" | "wandering_trader" => VILLAGER,
        "chicken" | "warm_chicken" | "cold_chicken" | "temperate_chicken" => CHICKEN,
        "spider" | "cave_spider" => SPIDER,
        "wolf" => WOLF,
        "cat" | "ocelot" => CAT,
        "horse" | "donkey" | "mule" | "skeleton_horse" | "zombie_horse" => HORSE,
        "bee" => BEE,
        "slime" | "magma_cube" => SLIME,
        "squid" | "glow_squid" => SQUID,
        "hoglin" | "zoglin" => HOGLIN,
        "llama" | "trader_llama" => LLAMA,
        _ => &[],
    }
}

/// The vanilla part an OptiFine name stands for in model `jem`.
pub fn vanilla<'a>(jem: &str, name: &'a str) -> &'a str {
    table(jem)
        .iter()
        .find(|(of, _)| *of == name)
        .map_or(name, |(_, v)| v)
}

/// Parts OptiFine names by their vanilla name in the models that have them.
const SAME: &[&str] = &[
    "head",
    "body",
    "left_arm",
    "right_arm",
    "left_leg",
    "right_leg",
    "hat",
    "ear",
    "left_ear",
    "right_ear",
    "left_sleeve",
    "right_sleeve",
    "left_pants",
    "right_pants",
    "jacket",
    "cloak",
    "nose",
    "arms",
    "tail",
    "tail2",
    "mane",
    "left_horn",
    "right_horn",
    "right_gills",
    "top_gills",
    "left_gills",
    "left_chest",
    "right_chest",
    "saddle",
    "right_wing",
    "left_wing",
    "stinger",
    "left_antenna",
    "right_antenna",
    "front_legs",
    "middle_legs",
    "back_legs",
    "left_eye",
    "right_eye",
    "mouth",
    "jaw",
    "cape",
];

/// Whether a vanilla part has an OptiFine name in this model: such a child
/// stays drawn when an entry replaces its parent's cubes (`hat` in `head`).
pub fn is_named(jem: &str, vanilla_part: &str) -> bool {
    SAME.contains(&vanilla_part) || table(jem).iter().any(|(_, v)| *v == vanilla_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_map_by_model_family() {
        assert_eq!(vanilla("cow", "leg1"), "right_hind_leg");
        assert_eq!(vanilla("cow_baby", "leg4"), "left_front_leg");
        assert_eq!(vanilla("zombie_baby_boots", "headwear"), "hat");
        assert_eq!(vanilla("spider", "body"), "body1");
        assert_eq!(vanilla("villager_baby", "bodywear"), "bb_main");
        assert_eq!(vanilla("villager", "bodywear"), "jacket");
        assert_eq!(vanilla("cow", "head"), "head");
        assert_eq!(vanilla("sheep_wool", "leg2"), "left_hind_leg");
        assert!(is_named("zombie", "hat"));
    }
}
