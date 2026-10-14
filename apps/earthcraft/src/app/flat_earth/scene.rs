//! Immutable scene topology and cached tessellation for the hidden illustration.
use super::{RINGS, SECTORS, STARS, hash01};
use eframe::egui::{self, Color32, Pos2, Rect, Shape, TextureId, Vec2};
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct View {
    pub rect: Rect,
    pub zoom: f32,
    pub offset: Vec2,
    pub yaw: f32,
    pub tilt: f32,
    pub texture: Option<TextureId>,
    pub pixels_per_point: f32,
}

#[derive(Default)]
pub(super) struct Scene {
    view: Option<View>,
    shapes: Vec<Shape>,
}
impl Scene {
    pub fn draw(&mut self, painter: &egui::Painter, view: View) {
        if self.view != Some(view) {
            let shapes = build(view);
            let clipped = shapes
                .into_iter()
                .map(|shape| egui::epaint::ClippedShape {
                    clip_rect: view.rect,
                    shape,
                })
                .collect();
            self.shapes = painter
                .ctx()
                .tessellate(clipped, view.pixels_per_point)
                .into_iter()
                .map(|primitive| match primitive.primitive {
                    egui::epaint::Primitive::Mesh(mesh) => Shape::mesh(mesh),
                    egui::epaint::Primitive::Callback(_) => {
                        unreachable!("illustration contains only meshes")
                    }
                })
                .collect();
            self.view = Some(view);
        }
        painter.extend(self.shapes.iter().cloned());
    }
}

struct Topology {
    vertices: Vec<(Vec2, Pos2)>,
    indices: Vec<u32>,
}
static TOPOLOGY: LazyLock<Topology> = LazyLock::new(|| {
    let mut vertices = Vec::with_capacity((RINGS + 1) * (SECTORS + 1));
    let mut indices = Vec::with_capacity(RINGS * SECTORS * 6);
    for ring in 0..=RINGS {
        let r = ring as f32 / RINGS as f32;
        for sector in 0..=SECTORS {
            let u = sector as f32 / SECTORS as f32;
            let angle = u * std::f32::consts::TAU;
            vertices.push((Vec2::new(r * angle.cos(), r * angle.sin()), Pos2::new(u, r)));
        }
    }
    for ring in 0..RINGS {
        for sector in 0..SECTORS {
            let inner = (ring * (SECTORS + 1) + sector) as u32;
            let outer = inner + (SECTORS + 1) as u32;
            indices.extend_from_slice(&[inner, inner + 1, outer + 1, inner, outer + 1, outer]);
        }
    }
    Topology { vertices, indices }
});

struct Projection {
    center: Pos2,
    radius: f32,
    yaw_sin: f32,
    yaw_cos: f32,
    tilt_sin: f32,
    tilt_cos: f32,
}
impl Projection {
    fn new(view: View) -> Self {
        let (yaw_sin, yaw_cos) = view.yaw.sin_cos();
        Self {
            center: view.rect.center() + view.offset + Vec2::new(0.0, view.rect.height() * 0.05),
            radius: (view.rect.width() * 0.31).min(view.rect.height() * 0.42) * view.zoom,
            yaw_sin,
            yaw_cos,
            tilt_sin: view.tilt.sin(),
            tilt_cos: view.tilt.cos(),
        }
    }
    fn point(&self, x: f32, y: f32, height: f32) -> Pos2 {
        let rx = x * self.yaw_cos - y * self.yaw_sin;
        let ry = x * self.yaw_sin + y * self.yaw_cos;
        Pos2::new(
            self.center.x + self.radius * rx,
            self.center.y + self.radius * (ry * self.tilt_sin - height * self.tilt_cos),
        )
    }
}

fn build(view: View) -> Vec<Shape> {
    let mut shapes = Vec::with_capacity(STARS + SECTORS + 2);
    let rect = view.rect;
    for i in 0..STARS as u32 {
        let x = rect.left() + rect.width() * hash01(i * 7 + 1);
        let y = rect.top() + rect.height() * hash01(i * 7 + 2);
        let depth = hash01(i * 7 + 3);
        let alpha = 90 + (depth * 150.0) as u8;
        let size = 0.35 + depth * 1.05;
        shapes.push(Shape::circle_filled(
            Pos2::new(x, y),
            size,
            Color32::from_rgba_unmultiplied(205, 223, 255, alpha),
        ));
    }
    let projection = Projection::new(view);
    let point = |x, y, z| projection.point(x, y, z);
    for j in 0..SECTORS {
        let a = (j as f32 + 0.5) * std::f32::consts::TAU / SECTORS as f32;
        let b = (j as f32 + 1.5) * std::f32::consts::TAU / SECTORS as f32;
        let p = [
            point(a.cos(), a.sin(), 0.0),
            point(b.cos(), b.sin(), 0.0),
            point(b.cos(), b.sin(), -0.10),
            point(a.cos(), a.sin(), -0.10),
        ];
        shapes.push(Shape::convex_polygon(
            p.to_vec(),
            Color32::from_rgb(117, 181, 206),
            egui::Stroke::NONE,
        ));
    }
    if let Some(texture) = view.texture {
        let mesh = egui::Mesh {
            texture_id: texture,
            vertices: TOPOLOGY
                .vertices
                .iter()
                .map(|(position, uv)| egui::epaint::Vertex {
                    pos: point(position.x, position.y, 0.0),
                    uv: *uv,
                    color: Color32::WHITE,
                })
                .collect(),
            indices: TOPOLOGY.indices.clone(),
        };
        shapes.push(Shape::mesh(mesh));
        let ice = (0..=SECTORS)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / SECTORS as f32;
                point(a.cos(), a.sin(), 0.001)
            })
            .collect();
        shapes.push(Shape::line(
            ice,
            egui::Stroke::new(3.0 * view.zoom, Color32::from_rgb(228, 247, 255)),
        ));
    }
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn view() -> View {
        View {
            rect: Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(800.0, 600.0)),
            zoom: 1.0,
            offset: Vec2::ZERO,
            yaw: 0.0,
            tilt: 0.62,
            texture: Some(TextureId::Managed(42)),
            pixels_per_point: 1.0,
        }
    }
    #[test]
    fn shared_disk_topology_keeps_the_uv_seam_and_valid_triangle_indices() {
        assert_eq!(TOPOLOGY.vertices.len(), (RINGS + 1) * (SECTORS + 1));
        assert_eq!(TOPOLOGY.indices.len(), RINGS * SECTORS * 6);
        for ring in 0..=RINGS {
            assert_eq!(TOPOLOGY.vertices[ring * (SECTORS + 1)].1.x, 0.0);
            assert_eq!(TOPOLOGY.vertices[ring * (SECTORS + 1) + SECTORS].1.x, 1.0);
        }
        assert!(
            TOPOLOGY
                .indices
                .iter()
                .all(|&index| (index as usize) < TOPOLOGY.vertices.len())
        );
    }
    #[test]
    fn projection_retains_the_original_corner_positions_and_rim_depth() {
        // Captured from disk_point at 432426f on macOS ARM64. Allow f32 libm rounding across platforms.
        let original: [[[u32; 2]; 4]; 3] = [
            [
                [1_137_508_352, 1_135_542_272],
                [1_143_242_752, 1_135_542_272],
                [1_137_508_352, 1_140_264_033],
                [1_137_508_352, 1_140_888_058],
            ],
            [
                [1_137_508_352, 1_135_542_272],
                [1_139_275_164, 1_136_086_305],
                [1_132_963_842, 1_135_753_781],
                [1_132_963_842, 1_136_237_862],
            ],
            [
                [1_137_508_352, 1_135_542_272],
                [3_287_509_880, 3_264_462_080],
                [1_146_125_694, 3_288_901_970],
                [1_146_125_694, 3_288_868_173],
            ],
        ];
        for (index, (yaw, tilt, zoom)) in [(0.0, 0.62, 1.0), (1.2, 0.12, 0.6), (-2.7, 1.55, 4.0)]
            .into_iter()
            .enumerate()
        {
            let projection = Projection::new(View {
                yaw,
                tilt,
                zoom,
                ..view()
            });
            for (point, (x, y, z)) in [
                (0.0, 0.0, 0.0),
                (1.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 1.0, -0.1),
            ]
            .into_iter()
            .enumerate()
            {
                let position = projection.point(x, y, z);
                for (actual, bits) in [position.x, position.y]
                    .into_iter()
                    .zip(original[index][point])
                {
                    let expected = f32::from_bits(bits);
                    assert!(
                        (actual - expected).abs() <= 4.0 * f32::EPSILON * expected.abs().max(1.0)
                    );
                }
            }
        }
    }
    #[test]
    fn stationary_scene_reuses_meshes_and_invalidates_view_texture_and_dpi() {
        let ctx = egui::Context::default();
        let mut scene = Scene::default();
        let original = view();
        crate::app::test_ui::run(&ctx, egui::RawInput::default(), |ui| {
            let painter = ui.painter();
            scene.draw(painter, original);
            let mesh = match &scene.shapes[0] {
                Shape::Mesh(mesh) => mesh.clone(),
                _ => panic!("cached mesh"),
            };
            scene.draw(painter, original);
            assert!(matches!(&scene.shapes[0],Shape::Mesh(next) if Arc::ptr_eq(&mesh,next)));
            for changed in [
                View {
                    offset: Vec2::new(1.0, 0.0),
                    ..original
                },
                View {
                    pixels_per_point: 2.0,
                    ..original
                },
                View {
                    texture: Some(TextureId::Managed(43)),
                    ..original
                },
            ] {
                scene.draw(painter, changed);
                assert!(matches!(&scene.shapes[0],Shape::Mesh(next) if !Arc::ptr_eq(&mesh,next)));
            }
        });
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    #[test]
    #[ignore]
    fn flat_scene_construction_time() {
        use std::{hint::black_box, time::Instant};
        let view = View {
            rect: Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(800.0, 600.0)),
            zoom: 1.0,
            offset: Vec2::ZERO,
            yaw: 1.2,
            tilt: 0.62,
            texture: Some(TextureId::Managed(42)),
            pixels_per_point: 1.0,
        };
        black_box(build(view));
        let mut rounds = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            for _ in 0..200 {
                black_box(build(black_box(view)));
            }
            rounds.push(start.elapsed().as_secs_f64() / 200.0);
        }
        rounds.sort_by(f64::total_cmp);
        eprintln!(
            "flat_scene median_us={:.3}, vertices={}, triangles={}",
            rounds[3] * 1e6,
            TOPOLOGY.vertices.len(),
            TOPOLOGY.indices.len() / 3
        );
    }
}
