//! Vanilla 26.3 MeshData.SortState / VertexSorting.byDistance.
//! Keep vertex order intact; only the quad indices change as the camera moves.

/// MeshData.decodeQuadCentroids uses the opposite corners, not all four vertices.
pub fn quad_center(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| (a[i] + b[i]) / 2.0)
}

#[derive(Default)]
pub struct Sorter {
    order: Vec<(u32, f32)>,
    indices: Vec<u32>,
}

impl Sorter {
    pub fn sort(&mut self, centers: &[[f32; 3]], camera: [f32; 3]) -> &[u32] {
        self.order.clear();
        self.order.extend(centers.iter().enumerate().map(|(i, c)| {
            let [x, y, z] = std::array::from_fn(|j| camera[j] - c[j]);
            (i as u32, x * x + y * y + z * z)
        }));
        // VertexSorting uses a stable merge sort, preserving mesh order for
        // ties: the quad index breaks them, without the stable sort's buffer.
        self.order
            .sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        self.indices.clear();
        for &(quad, _) in &self.order {
            let v = quad * 4;
            self.indices.extend([v, v + 1, v + 2, v + 2, v + 3, v]);
        }
        &self.indices
    }
}

/// Whether the quads lie in one horizontal plane without overlapping, as a
/// far tile's sea does: no pixel then blends two of them, so every order
/// draws the same.
pub fn flat_and_disjoint(quads: impl IntoIterator<Item = [[f32; 3]; 4]>) -> bool {
    let mut plane = None;
    let mut rects = Vec::new();
    for quad in quads {
        let y = *plane.get_or_insert(quad[0][1]);
        if quad.iter().any(|p| p[1] != y) {
            return false;
        }
        let mut rect = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
        for p in quad {
            rect.0 = [rect.0[0].min(p[0]), rect.0[1].min(p[2])];
            rect.1 = [rect.1[0].max(p[0]), rect.1[1].max(p[2])];
        }
        rects.push(rect);
    }
    disjoint(&mut rects)
}

/// Whether no two rectangles (min and max x/z corners) share any area: a
/// sweep along x, testing each against those still open there.
fn disjoint(rects: &mut [([f32; 2], [f32; 2])]) -> bool {
    rects.sort_unstable_by(|a, b| a.0[0].total_cmp(&b.0[0]));
    let mut open: Vec<([f32; 2], [f32; 2])> = Vec::new();
    for &(min, max) in rects.iter() {
        open.retain(|(_, end)| end[0] > min[0]);
        if open
            .iter()
            .any(|(start, end)| start[0] < max[0] && start[1] < max[1] && min[1] < end[1])
        {
            return false;
        }
        open.push((min, max));
    }
    true
}

pub struct SortState {
    centers: Vec<[f32; 3]>,
    last: Option<([i32; 3], [i32; 3])>,
    /// The quads are `flat_and_disjoint`: the first order serves every view.
    flat: bool,
}

impl SortState {
    pub fn new(centers: Vec<[f32; 3]>) -> Self {
        Self {
            centers,
            last: None,
            flat: false,
        }
    }

    pub fn is_sorted(&self) -> bool {
        self.last.is_some()
    }

    /// Marks the quads `flat_and_disjoint`: `update_lod` sorts them once.
    pub fn set_flat(&mut self, flat: bool) {
        self.flat = flat;
    }

    /// Large tiles cannot use the section viewpoint shortcut: the camera may
    /// cross a quad's plane while remaining on the same side of the first 16³.
    pub fn update_lod<'a>(
        &mut self,
        camera: [f64; 3],
        origin: [i32; 3],
        sorter: &'a mut Sorter,
    ) -> Option<&'a [u32]> {
        let block = camera.map(|v| v.floor() as i32);
        if self
            .last
            .is_some_and(|(last, _)| self.flat || last == block)
        {
            return None;
        }
        self.last = Some((block, [0; 3]));
        Some(sorter.sort(
            &self.centers,
            std::array::from_fn(|i| (camera[i] - f64::from(origin[i])) as f32),
        ))
    }

    /// TranslucencyPointOfView and ResortTransparencyTask: diagonal views keep
    /// their ordering until the point of view changes; aligned views also
    /// resort on crossing a block boundary. Sub-block movement needs no upload.
    pub fn update<'a>(
        &mut self,
        camera: [f64; 3],
        origin: [i32; 3],
        sorter: &'a mut Sorter,
    ) -> Option<&'a [u32]> {
        let block = camera.map(|v| v.floor() as i32);
        let pov = std::array::from_fn(|i| ((block[i] >> 4) - (origin[i] >> 4)).clamp(-1, 1));
        if let Some((last_block, last_pov)) = self.last
            && last_pov == pov
            && (last_block == block || !pov.contains(&0))
        {
            return None;
        }
        self.last = Some((block, pov));
        Some(sorter.sort(
            &self.centers,
            std::array::from_fn(|i| (camera[i] - f64::from(origin[i])) as f32),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_distances_keep_mesh_order_and_triangle_winding() {
        let mut sorter = Sorter::default();
        assert_eq!(
            sorter.sort(
                &[[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
                [0.0; 3]
            ),
            &[8, 9, 10, 10, 11, 8, 0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4]
        );
        assert!(sorter.sort(&[], [0.0; 3]).is_empty());
    }

    #[test]
    fn index_tie_breaks_order_as_the_stable_sort_did() {
        // Few distinct distances: most quads tie with others.
        let centers: Vec<[f32; 3]> = (0..500u32)
            .map(|i| [(i * 7 % 5) as f32, 0.0, (i * 13 % 3) as f32])
            .collect();
        for camera in [[0.0; 3], [2.0, 1.0, 1.0], [-3.0, 0.5, 9.0]] {
            let mut stable: Vec<(u32, f32)> = centers
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let [x, y, z] = std::array::from_fn(|j| camera[j] - c[j]);
                    (i as u32, x * x + y * y + z * z)
                })
                .collect();
            stable.sort_by(|a, b| b.1.total_cmp(&a.1));
            let expected: Vec<u32> = stable
                .iter()
                .flat_map(|&(q, _)| [q * 4, q * 4 + 1, q * 4 + 2, q * 4 + 2, q * 4 + 3, q * 4])
                .collect();
            assert_eq!(Sorter::default().sort(&centers, camera), expected);
        }
    }

    #[test]
    fn one_sheet_of_water_sorts_once() {
        let quad = |x0: f32, z0: f32, x1: f32, z1: f32, y: f32| {
            [[x0, y, z1], [x1, y, z1], [x1, y, z0], [x0, y, z0]]
        };
        let sheet = [quad(0., 0., 16., 8., 62.9), quad(0., 8., 4., 16., 62.9)];
        assert!(flat_and_disjoint(sheet));
        assert!(flat_and_disjoint(std::iter::empty()));
        // Another height, an overlap, or a face over the same area (two
        // layers of one face) still need sorting.
        assert!(!flat_and_disjoint([sheet[0], quad(0., 8., 4., 16., 61.9)]));
        assert!(!flat_and_disjoint([sheet[0], quad(15., 7., 20., 9., 62.9)]));
        assert!(!flat_and_disjoint([sheet[1], sheet[1]]));
        assert!(!flat_and_disjoint([[[0., 0., 0.], [1., 1., 0.], [1., 1., 1.], [0., 0., 1.]]]));
        // Sharing an edge or a corner is no overlap.
        assert!(flat_and_disjoint([
            quad(0., 0., 1., 1., 0.),
            quad(1., 0., 2., 1., 0.),
            quad(1., 1., 2., 2., 0.),
            quad(0., 1., 1., 2., 0.),
        ]));
        let centers = sheet.map(|q| quad_center(q[0], q[2])).to_vec();
        let mut sorter = Sorter::default();
        let mut state = SortState::new(centers.clone());
        state.set_flat(true);
        assert!(state.update_lod([0.5; 3], [0; 3], &mut sorter).is_some());
        assert!(state.update_lod([90.5; 3], [0; 3], &mut sorter).is_none());
        let mut state = SortState::new(centers);
        assert!(state.update_lod([0.5; 3], [0; 3], &mut sorter).is_some());
        assert!(state.update_lod([90.5; 3], [0; 3], &mut sorter).is_some());
    }

    #[test]
    fn movement_reverses_faces_without_moving_vertices() {
        let centers = [[0.5, 0.5, 0.0], [0.5, 0.5, 1.0]];
        let mut sorter = Sorter::default();
        assert_eq!(sorter.sort(&centers, [0.5, 0.5, -2.0])[0], 4);
        assert_eq!(sorter.sort(&centers, [0.5, 0.5, 3.0])[0], 0);
    }

    #[test]
    fn cached_viewpoint_handles_negative_and_large_coordinates() {
        let origin = [-30_000_000, -64, 30_000_000];
        let mut state = SortState::new(vec![[0.5; 3]]);
        let mut sorter = Sorter::default();
        let camera = [-29_999_999.5, -63.5, 30_000_000.5];
        assert!(state.update(camera, origin, &mut sorter).is_some());
        assert!(state.update(camera, origin, &mut sorter).is_none());
        assert!(
            state
                .update([-29_999_999.1, -63.1, 30_000_000.9], origin, &mut sorter)
                .is_none()
        );
        assert!(
            state
                .update([-29_999_998.9, -63.1, 30_000_000.9], origin, &mut sorter)
                .is_some()
        );
    }
    #[test]
    fn large_tiles_resort_inside_the_same_section_viewpoint() {
        let mut state = SortState::new(vec![[32.0, 0.0, 0.0], [96.0, 0.0, 0.0]]);
        let mut sorter = Sorter::default();
        assert_eq!(
            state
                .update_lod([48.0, 20.0, 20.0], [0; 3], &mut sorter)
                .unwrap()[0],
            4
        );
        assert!(
            state
                .update_lod([48.5, 20.0, 20.0], [0; 3], &mut sorter)
                .is_none()
        );
        // Both views have positive x/y/z signs relative to the first section,
        // so the section-sized shortcut would incorrectly keep the old order.
        assert_eq!(
            state
                .update_lod([80.0, 20.0, 20.0], [0; 3], &mut sorter)
                .unwrap()[0],
            0
        );
    }
}
