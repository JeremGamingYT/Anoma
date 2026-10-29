//! A `.jem` placed on its vanilla model, as EMF does it: each top-level entry
//! becomes a custom part under the vanilla part it names (which keeps its own
//! pivot and its vanilla pose), with the entry's boxes and submodels below it;
//! the entries' animation lines compile to one program over the result.
use super::{
    cube::{Cube, FaceUvs, box_uvs},
    jem::{JemDef, PartDef},
    names,
    program::Program,
};
use crate::mobs::Part;
use std::collections::HashSet;

/// A model ready to pose and draw.
#[derive(Clone, Debug)]
pub struct CemModel {
    pub part: std::sync::Arc<Part>,
    pub program: Program,
}

/// Fills the fields an entry leaves unset from its `.jpm` (`"model"`).
fn merge(entry: &PartDef, jpm: &PartDef) -> PartDef {
    let mut out = entry.clone();
    if out.submodels.is_empty() {
        out.submodels = jpm.submodels.clone();
    }
    if out.texture_size.is_none() {
        out.texture_size = jpm.texture_size;
    }
    if out.texture.is_none() {
        out.texture = jpm.texture.clone();
    }
    if out.invert == [false; 3] {
        out.invert = jpm.invert;
    }
    if out.mirror == [false; 2] {
        out.mirror = jpm.mirror;
    }
    if out.translate == [0.0; 3] {
        out.translate = jpm.translate;
    }
    if out.rotate == [0.0; 3] {
        out.rotate = jpm.rotate;
    }
    if out.boxes.is_empty() {
        out.boxes = jpm.boxes.clone();
    }
    if out.scale == 1.0 {
        out.scale = jpm.scale;
    }
    if out.animations.is_empty() {
        out.animations = jpm.animations.clone();
    }
    out
}

/// A custom part and its submodels, named `EMF_<id>` (unique in `taken`).
fn custom(def: &PartDef, texture: [f32; 2], taken: &mut HashSet<String>) -> Part {
    let texture = def.texture_size.unwrap_or(texture);
    let sign = |a: usize| if def.invert[a] { -1.0 } else { 1.0 };
    let mut name = format!("EMF_{}", def.id.as_deref().unwrap_or("part"));
    while !taken.insert(name.clone()) {
        name.push('#');
    }
    let mut polygons = Vec::new();
    for b in &def.boxes {
        let [x, y, z, w, h, d] = b.coordinates;
        let min = [
            if def.invert[0] { -x - w } else { x },
            if def.invert[1] { -y - h } else { y },
            if def.invert[2] { -z - d } else { z },
        ];
        let (uvs, per_face): (FaceUvs, bool) = match b.texture_offset {
            Some([u, v]) => (box_uvs([u.trunc(), v.trunc()], [w, h, d]), false),
            // EMF's faces for per-face UVs: the y0 face takes uvUp, x1 uvWest.
            None => {
                let f = &b.face_uvs;
                ([f[1], f[0], f[4], f[3], f[2], f[5]], true)
            }
        };
        let cube = Cube {
            min,
            size: [w, h, d],
            grow: b.size_add,
            mirror: def.mirror[0] && !per_face,
            per_face,
        };
        polygons.extend(cube.polygons(&uvs, texture));
    }
    let children = def
        .submodels
        .iter()
        .map(|s| custom(s, texture, taken))
        .collect();
    let r = |a: usize| (sign(a) * def.rotate[a]).to_radians();
    Part {
        name,
        pose: [
            sign(0) * def.translate[0],
            sign(1) * def.translate[1],
            sign(2) * def.translate[2],
            r(0),
            r(1),
            r(2),
            def.scale,
            def.scale,
            def.scale,
        ],
        polygons,
        children,
    }
}

fn find_mut<'a>(part: &'a mut Part, name: &str) -> Option<&'a mut Part> {
    if part.name == name {
        return Some(part);
    }
    part.children.iter_mut().find_map(|c| find_mut(c, name))
}
fn names_in(part: &Part, out: &mut HashSet<String>) {
    out.insert(part.name.clone());
    for c in &part.children {
        names_in(c, out);
    }
}

/// Replaces whole words `word` in `text` (`this`, `part`).
fn replace_word(text: &str, word: &str, with: &str) -> String {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.';
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(word) {
        let before = rest[..i].chars().next_back();
        let after = rest[i + word.len()..].chars().next();
        out.push_str(&rest[..i]);
        // `this.rx` is a word followed by a dot; `var.this` is not `this`.
        if before.is_none_or(|c| !is_word(c)) && after.is_none_or(|c| !(is_word(c) && c != '.')) {
            out.push_str(with);
        } else {
            out.push_str(word);
        }
        rest = &rest[i + word.len()..];
    }
    out.push_str(rest);
    out
}

/// Places `def` (model `jem`, e.g. `cow`) on `vanilla`; `jpm` reads a part
/// file an entry names.
pub fn build(
    jem: &str,
    def: &JemDef,
    vanilla: &Part,
    jpm: &mut dyn FnMut(&str) -> Option<PartDef>,
) -> CemModel {
    let texture = def.texture_size.unwrap_or([64.0, 32.0]);
    let mut tree = vanilla.clone();
    let root = tree.name.clone();
    let mut taken = HashSet::new();
    names_in(&tree, &mut taken);
    // Lines grouped by the vanilla part their entry targets, groups in order
    // of first appearance (EMF's LinkedHashMap).
    let mut groups: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for entry in &def.models {
        let entry = match entry.model.as_deref().and_then(&mut *jpm) {
            Some(file) => merge(entry, &file),
            None => entry.clone(),
        };
        let Some(part) = entry.part.as_deref() else {
            continue;
        };
        let target = if part == "root" {
            root.clone()
        } else {
            names::vanilla(jem, part).to_owned()
        };
        let made = custom(&entry, texture, &mut taken);
        let this = made.name.clone();
        let Some(vanilla_part) = find_mut(&mut tree, &target) else {
            continue;
        };
        if !entry.attach {
            vanilla_part.polygons.clear();
            vanilla_part
                .children
                .retain(|c| c.name.starts_with("EMF_") || names::is_named(jem, &c.name));
        }
        vanilla_part.children.push(made);
        let lines = entry.animations.iter().flatten().map(|(k, e)| {
            let fix = |s: &str| replace_word(&replace_word(s, "this", &this), "part", part);
            (fix(k), fix(e))
        });
        match groups.iter_mut().find(|(g, _)| *g == target) {
            Some((_, g)) => g.extend(lines),
            None => groups.push((target, lines.collect())),
        }
    }
    let mut all = HashSet::new();
    names_in(&tree, &mut all);
    let resolve = |name: &str| -> Option<String> {
        // `a:b:c` addresses the last segment.
        let name = name.rsplit(':').next().unwrap_or(name);
        if name == "root" {
            return Some(root.clone());
        }
        let mapped = names::vanilla(jem, name);
        if all.contains(mapped) && !mapped.starts_with("EMF_") {
            return Some(mapped.to_owned());
        }
        let custom = format!("EMF_{name}");
        if all.contains(&custom) {
            return Some(custom);
        }
        (name.starts_with("EMF_") && all.contains(name)).then(|| name.to_owned())
    };
    let lines: Vec<(String, String)> = groups.into_iter().flat_map(|(_, l)| l).collect();
    let mut program = Program::compile(&lines, &resolve);
    // EMF drops every animation of a model when one line fails to parse.
    if !program.errors.is_empty() {
        log::warn!(
            "Anoma: {jem}.jem animations off: {} ({})",
            program.errors[0].0,
            program.errors[0].1
        );
        program = Program::default();
    }
    CemModel {
        part: std::sync::Arc::new(tree),
        program,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cem::jem::Value;

    fn vanilla_cow() -> Part {
        let leg = |x: f32, z: f32| Part {
            name: String::new(),
            pose: [x, 12.0, z, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            polygons: Vec::new(),
            children: Vec::new(),
        };
        let mut root = leg(0.0, 0.0);
        root.name = "root".into();
        root.pose = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        for (name, x, z) in [("right_hind_leg", -4.0, 7.0), ("head", 0.0, -8.0)] {
            let mut p = leg(x, z);
            p.name = name.into();
            p.polygons = crate::cem::cube::Cube {
                min: [-2.0, 0.0, -2.0],
                size: [4.0, 12.0, 4.0],
                grow: 0.0,
                mirror: false,
                per_face: false,
            }
            .polygons(&box_uvs([0.0, 16.0], [4.0, 12.0, 4.0]), [64.0, 64.0]);
            root.children.push(p);
        }
        root
    }

    #[test]
    fn entries_hang_under_their_vanilla_part_and_lines_find_their_parts() {
        let json = br#"{"textureSize":[64,64],"models":[
            {"part":"leg1","id":"leg1","invertAxis":"xy","translate":[-4,0,-7],
             "boxes":[{"coordinates":[2,0,5,4,12,4],"textureOffset":[0,16]}],
             "submodels":[{"id":"foot","invertAxis":"xy","translate":[0,1,0]}]},
            {"part":"root","id":"root","model":"cow_animations.jpm"}]}"#;
        let def = JemDef::parse(&Value::parse(json).unwrap());
        let jpm =
            br#"{"animations":[{"leg1.ty":"24","var.t":"this.ty","foot.rx":"1","part.tz":"7"}]}"#;
        let model = build("cow", &def, &vanilla_cow(), &mut |name| {
            assert_eq!(name, "cow_animations.jpm");
            Some(PartDef::parse(&Value::parse(jpm).unwrap()))
        });
        let leg = model
            .part
            .children
            .iter()
            .find(|c| c.name == "right_hind_leg")
            .unwrap();
        // The vanilla cubes went, the custom part came, at translate' = (4, 0, -7).
        assert_eq!(leg.children.len(), 1);
        let custom = &leg.children[0];
        assert_eq!(custom.name, "EMF_leg1");
        assert_eq!(&custom.pose[..3], &[4.0, 0.0, -7.0]);
        assert_eq!(custom.children[0].name, "EMF_foot");
        assert_eq!(custom.polygons.len(), 6);
        // Box min after the invert: (-6, -12, 5).
        let xs: Vec<f32> = custom
            .polygons
            .iter()
            .flat_map(|p| p.vertices.map(|v| v[0]))
            .collect();
        assert_eq!(xs.iter().cloned().fold(f32::INFINITY, f32::min), -6.0);
        assert!(model.program.errors.is_empty());
        // `this` and `part` in the root entry's file: its custom part and root.
        assert_eq!(
            model.program.parts,
            ["right_hind_leg", "EMF_root", "EMF_foot", "root"]
        );
    }

    #[test]
    fn this_and_part_are_whole_words() {
        assert_eq!(
            replace_word("this.rx+var.this+thisx", "this", "EMF_a"),
            "EMF_a.rx+var.this+thisx"
        );
        assert_eq!(replace_word("part.tz", "part", "leg1"), "leg1.tz");
    }
}
