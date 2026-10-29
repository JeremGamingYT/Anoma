//! `ModelPart.Cube`: a box's six polygons with the box UV layout of a texture
//! offset, or each face's own UV rectangle, as vanilla and the CEM mods build
//! them.
use crate::mobs::Polygon;

/// The faces in vanilla's order (`Direction`), with their normals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Down,
    Up,
    West,
    North,
    East,
    South,
}
impl Face {
    pub const ALL: [Face; 6] = [
        Face::Down,
        Face::Up,
        Face::West,
        Face::North,
        Face::East,
        Face::South,
    ];
    fn normal(self) -> [f32; 3] {
        match self {
            Face::Down => [0.0, -1.0, 0.0],
            Face::Up => [0.0, 1.0, 0.0],
            Face::West => [-1.0, 0.0, 0.0],
            Face::North => [0.0, 0.0, -1.0],
            Face::East => [1.0, 0.0, 0.0],
            Face::South => [0.0, 0.0, 1.0],
        }
    }
}

/// A box in part space (pixels), grown by `grow` on every side.
#[derive(Clone, Copy, Debug)]
pub struct Cube {
    pub min: [f32; 3],
    pub size: [f32; 3],
    pub grow: f32,
    pub mirror: bool,
    /// Per-face UVs: EMF builds the top and bottom faces from other corners
    /// than the box layout does, which turns their texture a quarter.
    pub per_face: bool,
}

/// Each face's UV rectangle [u0, v0, u1, v1] in texels (None: no face).
pub type FaceUvs = [Option<[f32; 4]>; 6];

/// The UV rectangles the texture offset `(u, v)` gives a box of `size`.
pub fn box_uvs(offset: [f32; 2], size: [f32; 3]) -> FaceUvs {
    let [w, h, d] = size;
    let (u, v) = (offset[0], offset[1]);
    let (u0, u1, u2, u22, u3, u4) = (
        u,
        u + d,
        u + d + w,
        u + d + w + w,
        u + d + w + d,
        u + d + w + d + w,
    );
    let (v0, v1, v2) = (v, v + d, v + d + h);
    [
        Some([u1, v0, u2, v1]),
        Some([u2, v1, u22, v0]),
        Some([u0, v1, u1, v2]),
        Some([u1, v1, u2, v2]),
        Some([u2, v1, u3, v2]),
        Some([u3, v1, u4, v2]),
    ]
}

impl Cube {
    /// The polygons, UVs normalized by the texture size.
    pub fn polygons(&self, uvs: &FaceUvs, texture: [f32; 2]) -> Vec<Polygon> {
        let g = self.grow;
        let (mut x0, y0, z0) = (self.min[0] - g, self.min[1] - g, self.min[2] - g);
        let (mut x1, y1, z1) = (
            self.min[0] + self.size[0] + g,
            self.min[1] + self.size[1] + g,
            self.min[2] + self.size[2] + g,
        );
        if self.mirror {
            std::mem::swap(&mut x0, &mut x1);
        }
        let c = [
            [x0, y0, z0],
            [x1, y0, z0],
            [x1, y1, z0],
            [x0, y1, z0],
            [x0, y0, z1],
            [x1, y0, z1],
            [x1, y1, z1],
            [x0, y1, z1],
        ];
        Face::ALL
            .iter()
            .zip(uvs)
            .filter_map(|(&face, uv)| {
                let [u0, v0, u1, v1] = (*uv)?;
                let corners = match face {
                    Face::Down if self.per_face => [0, 1, 5, 4],
                    Face::Up if self.per_face => [7, 6, 2, 3],
                    Face::Down => [5, 4, 0, 1],
                    Face::Up => [2, 3, 7, 6],
                    Face::West => [0, 4, 7, 3],
                    Face::North => [1, 0, 3, 2],
                    Face::East => [5, 1, 2, 6],
                    Face::South => [4, 5, 6, 7],
                };
                let (tw, th) = (texture[0], texture[1]);
                let uv = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
                let mut vertices: [[f32; 5]; 4] = std::array::from_fn(|i| {
                    let p = c[corners[i]];
                    [p[0], p[1], p[2], uv[i][0] / tw, uv[i][1] / th]
                });
                let mut normal = face.normal();
                if self.mirror {
                    vertices.reverse();
                    normal[0] = -normal[0];
                }
                Some(Polygon { normal, vertices })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_matches_vanillas_cube() {
        // CowModel's head: addBox(-4, -4, -6, 8, 8, 6) at texOffs(0, 0), 64x64.
        let cube = Cube {
            min: [-4.0, -4.0, -6.0],
            size: [8.0, 8.0, 6.0],
            grow: 0.0,
            mirror: false,
            per_face: false,
        };
        let polys = cube.polygons(&box_uvs([0.0, 0.0], cube.size), [64.0, 64.0]);
        assert_eq!(polys.len(), 6);
        // The first two vertices of the exported down face.
        assert_eq!(polys[0].vertices[0], [4.0, -4.0, 0.0, 0.21875, 0.0]);
        assert_eq!(polys[0].vertices[1], [-4.0, -4.0, 0.0, 0.09375, 0.0]);
        assert_eq!(polys[0].normal, [0.0, -1.0, 0.0]);
        // A mirrored left leg: addBox(-2, 0, -2, 4, 12, 4) at texOffs(0, 16).
        let leg = Cube {
            min: [-2.0, 0.0, -2.0],
            size: [4.0, 12.0, 4.0],
            grow: 0.0,
            mirror: true,
            per_face: false,
        };
        let polys = leg.polygons(&box_uvs([0.0, 16.0], leg.size), [64.0, 64.0]);
        assert_eq!(polys[0].vertices[0], [-2.0, 0.0, -2.0, 0.125, 0.3125]);
        assert_eq!(polys[0].vertices[1], [2.0, 0.0, -2.0, 0.0625, 0.3125]);
    }
}
