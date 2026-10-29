//! OptiFine CEM / EMF custom entity models: `.jem` geometry over the vanilla
//! model parts and the animation expressions that pose them each frame (the
//! format of Fresh Animations and the packs built for those mods).
pub mod cube;
pub mod expr;
pub mod jem;
pub mod model;
pub mod names;
pub mod program;
pub mod runtime;

use crate::mobs::Part;
use jem::{JemDef, PartDef, Value};
use model::CemModel;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// The `.jem` file name of a model layer (`cow#main` is `cow`, `sheep#wool`
/// `sheep_wool`), and the name to fall back on (`cow_baby` takes `cow`).
pub fn jem_names(layer: &str) -> (String, Option<String>) {
    let (path, layer) = layer.split_once('#').unwrap_or((layer, "main"));
    let path = path.trim_start_matches("minecraft:");
    let name = match (path, layer) {
        (p, "main") => p.to_owned(),
        ("creeper", "armor") => "creeper_charge".to_owned(),
        (p, l) => format!("{p}_{l}"),
    };
    let fallback = (name.contains("_baby")
        && !["horse", "donkey", "mule"].iter().any(|h| name.starts_with(h)))
    .then(|| name.replacen("_baby", "", 1));
    (name, fallback)
}

/// The custom models of a resource root, built the first time each model
/// layer is drawn.
#[derive(Default)]
pub struct Library {
    /// `assets/minecraft/emf/cem` and `optifine/cem` that exist, in the
    /// order they are searched.
    dirs: Vec<PathBuf>,
    assets: PathBuf,
    built: Mutex<HashMap<String, Option<Arc<CemModel>>>>,
}

impl Library {
    pub fn load(root: &Path) -> Self {
        let assets = root.join("assets");
        let dirs: Vec<PathBuf> = ["emf/cem", "optifine/cem"]
            .iter()
            .map(|d| assets.join("minecraft").join(d))
            .filter(|d| d.is_dir())
            .collect();
        if !dirs.is_empty() {
            log::info!("Anoma: custom entity models from {}", dirs[0].display());
        }
        Self {
            dirs,
            assets,
            built: Mutex::default(),
        }
    }

    /// The jem `name`, if a pack has one: its path.
    fn find(&self, name: &str) -> Option<PathBuf> {
        self.dirs.iter().find_map(|d| {
            [d.join(name).join(format!("{name}.jem")), d.join(format!("{name}.jem"))]
                .into_iter()
                .find(|p| p.is_file())
        })
    }

    /// A file a jem names (`cow_animations.jpm`), as EMF resolves it.
    fn resolve(&self, jem: &Path, name: &str) -> PathBuf {
        let name = if name.ends_with(".jpm") { name.to_owned() } else { format!("{name}.jpm") };
        let folder = jem.parent().unwrap_or(Path::new("."));
        if let Some((ns, path)) = name.split_once(':') {
            self.assets.join(ns).join(path)
        } else if let Some(rest) = name.strip_prefix("./") {
            folder.join(rest)
        } else if let Some(rest) = name.strip_prefix("~/") {
            self.assets.join("minecraft/optifine").join(rest)
        } else if !name.contains('/') {
            folder.join(name)
        } else {
            self.assets.join("minecraft").join(name)
        }
    }

    /// The custom model for vanilla model layer `layer` (`cow#main`) drawn
    /// with `vanilla`, or None when no pack has one.
    pub fn model(&self, layer: &str, vanilla: &Part) -> Option<Arc<CemModel>> {
        if self.dirs.is_empty() {
            return None;
        }
        let mut built = self.built.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(m) = built.get(layer) {
            return m.clone();
        }
        let made = self.build(layer, vanilla);
        built.insert(layer.to_owned(), made.clone());
        made
    }

    fn build(&self, layer: &str, vanilla: &Part) -> Option<Arc<CemModel>> {
        let (name, fallback) = jem_names(layer);
        let (name, path) = std::iter::once(name)
            .chain(fallback)
            .find_map(|n| Some((n.clone(), self.find(&n)?)))?;
        let read = |p: &Path| std::fs::read(p).ok().and_then(|b| Value::parse(&b).ok());
        let Some(json) = read(&path) else {
            log::warn!("Anoma: {} is not valid JSON", path.display());
            return None;
        };
        let def = JemDef::parse(&json);
        let model = model::build(&name, &def, vanilla, &mut |file| {
            read(&self.resolve(&path, file)).map(|v| PartDef::parse(&v))
        });
        Some(Arc::new(model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_name_their_jem_files() {
        assert_eq!(jem_names("cow#main"), ("cow".into(), None));
        assert_eq!(jem_names("sheep#wool"), ("sheep_wool".into(), None));
        assert_eq!(jem_names("creeper#armor").0, "creeper_charge");
        assert_eq!(
            jem_names("zombie_baby#helmet"),
            ("zombie_baby_helmet".into(), Some("zombie_helmet".into()))
        );
        assert_eq!(jem_names("horse_baby#main").1, None);
    }
}
