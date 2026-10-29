//! The `.jem` / `.jpm` files as written: a model's parts, their boxes,
//! submodels and animation lines, before any of it is placed or compiled.
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// JSON that keeps every object's keys in file order: animation lines run in
/// the order they are written, and `serde_json::Value` sorts them.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}
static NULL: Value = Value::Null;
impl std::ops::Index<&str> for Value {
    type Output = Value;
    fn index(&self, key: &str) -> &Value {
        match self {
            Value::Obj(o) => o.iter().find(|(k, _)| k == key).map_or(&NULL, |(_, v)| v),
            _ => &NULL,
        }
    }
}
impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_object(&self) -> Option<&Vec<(String, Value)>> {
        match self {
            Value::Obj(o) => Some(o),
            _ => None,
        }
    }
    /// The text of a scalar, as an expression written as a number or `true`.
    fn scalar_text(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Num(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => String::new(),
        }
    }
    pub fn parse(bytes: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON")
            }
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_bool<E>(self, b: bool) -> Result<Value, E> {
                Ok(Value::Bool(b))
            }
            fn visit_i64<E>(self, n: i64) -> Result<Value, E> {
                Ok(Value::Num(n as f64))
            }
            fn visit_u64<E>(self, n: u64) -> Result<Value, E> {
                Ok(Value::Num(n as f64))
            }
            fn visit_f64<E>(self, n: f64) -> Result<Value, E> {
                Ok(Value::Num(n))
            }
            fn visit_str<E>(self, s: &str) -> Result<Value, E> {
                Ok(Value::Str(s.to_owned()))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
                let mut out = Vec::new();
                while let Some(v) = a.next_element()? {
                    out.push(v);
                }
                Ok(Value::Arr(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
                let mut out: Vec<(String, Value)> = Vec::new();
                while let Some((k, v)) = a.next_entry::<String, Value>()? {
                    // A repeated key keeps its first place and its last value.
                    match out.iter_mut().find(|(o, _)| *o == k) {
                        Some(slot) => slot.1 = v,
                        None => out.push((k, v)),
                    }
                }
                Ok(Value::Obj(out))
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoxDef {
    /// [x, y, z, width, height, depth] in the file's axes.
    pub coordinates: [f32; 6],
    pub texture_offset: Option<[f32; 2]>,
    /// uvDown, uvUp, uvWest, uvNorth, uvEast, uvSouth ([u0, v0, u1, v1]).
    pub face_uvs: [Option<[f32; 4]>; 6],
    pub size_add: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PartDef {
    /// The vanilla part a top-level entry stands for.
    pub part: Option<String>,
    pub id: Option<String>,
    pub invert: [bool; 3],
    pub translate: [f32; 3],
    /// Degrees.
    pub rotate: [f32; 3],
    pub scale: f32,
    pub mirror: [bool; 2],
    /// Keep the vanilla part's own cubes and add these.
    pub attach: bool,
    pub texture: Option<String>,
    pub texture_size: Option<[f32; 2]>,
    /// A `.jpm` file this entry takes its content from.
    pub model: Option<String>,
    pub boxes: Vec<BoxDef>,
    pub submodels: Vec<PartDef>,
    /// Each animation object's lines in file order: (target, expression).
    pub animations: Vec<Vec<(String, String)>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JemDef {
    pub texture: Option<String>,
    pub texture_size: Option<[f32; 2]>,
    pub shadow_size: Option<f32>,
    pub models: Vec<PartDef>,
}

fn floats<const N: usize>(v: &Value) -> Option<[f32; N]> {
    let a = v.as_array()?;
    let mut out = [0.0; N];
    for (o, x) in out.iter_mut().zip(a) {
        *o = x.as_f64().or_else(|| x.as_str()?.trim().parse().ok())? as f32;
    }
    (a.len() >= N).then_some(out)
}
fn text(v: &Value) -> Option<String> {
    v.as_str().map(str::to_owned)
}

impl BoxDef {
    fn parse(v: &Value) -> Option<Self> {
        let face = |k: &str| floats::<4>(&v[k]);
        Some(Self {
            coordinates: floats(&v["coordinates"])?,
            texture_offset: floats(&v["textureOffset"]),
            face_uvs: [
                face("uvDown"),
                face("uvUp"),
                face("uvWest"),
                face("uvNorth").or_else(|| face("uvFront")),
                face("uvEast"),
                face("uvSouth").or_else(|| face("uvBack")),
            ],
            size_add: v["sizeAdd"].as_f64().unwrap_or(0.0) as f32,
        })
    }
}

impl PartDef {
    pub fn parse(v: &Value) -> Self {
        let invert = v["invertAxis"].as_str().unwrap_or("");
        let mirror = v["mirrorTexture"].as_str().unwrap_or("");
        Self {
            part: text(&v["part"]),
            id: text(&v["id"]),
            invert: ['x', 'y', 'z'].map(|a| invert.contains(a)),
            translate: floats(&v["translate"]).unwrap_or_default(),
            rotate: floats(&v["rotate"]).unwrap_or_default(),
            scale: v["scale"].as_f64().unwrap_or(1.0) as f32,
            mirror: ['u', 'v'].map(|a| mirror.contains(a)),
            attach: v["attach"].as_bool().unwrap_or(false),
            texture: text(&v["texture"]),
            texture_size: floats(&v["textureSize"]),
            model: text(&v["model"]),
            boxes: v["boxes"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(BoxDef::parse)
                .collect(),
            submodels: v["submodels"]
                .as_array()
                .into_iter()
                .flatten()
                .map(PartDef::parse)
                .collect(),
            animations: animations(&v["animations"]),
        }
    }
}

/// Each animation object's lines in file order.
pub fn animations(v: &Value) -> Vec<Vec<(String, String)>> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .map(|o| {
            o.iter()
                .map(|(k, e)| (k.trim().to_owned(), e.scalar_text()))
                .collect()
        })
        .collect()
}

impl JemDef {
    pub fn parse(v: &Value) -> Self {
        Self {
            texture: text(&v["texture"]),
            texture_size: floats(&v["textureSize"]),
            shadow_size: v["shadowSize"].as_f64().map(|s| s as f32),
            models: v["models"]
                .as_array()
                .into_iter()
                .flatten()
                .map(PartDef::parse)
                .collect(),
        }
    }
}
