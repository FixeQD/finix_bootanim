use std::path::Path as FsPath;

use eros::{Context, bail};
use image::{Rgba, RgbaImage, imageops};
use tiny_skia::{
    Color, FillRule, FilterQuality, GradientStop, IntSize, LinearGradient, Paint, Path,
    PathBuilder, Pattern, Pixmap, Point, SpreadMode, Stroke, Transform,
};

use crate::timeline::*;

/// Colours of the logo's red gradient
const RED: [(f32, [u8; 3]); 3] = [
    (0.0, [0xd3, 0x1f, 0x30]),
    (0.2435, [0xde, 0x30, 0x40]),
    (1.0, [0xe8, 0x40, 0x50]),
];
const ACCENT: [u8; 3] = [0xde, 0x30, 0x40];

type Stops = Vec<(f32, [u8; 3], f32)>;

struct Arm {
    path: Path,
    rule: FillRule,
    to_content: Transform,
    gradient: (Point, Point, Stops, Transform),
    dir: [f32; 2],
    red: bool,
}

pub struct Scene {
    arms: Vec<Arm>, // clockwise like the order of the tracks
    tl: Timeline,
    reveal: Pixmap,
    sc: f32,
    size: (u32, u32),
}

fn collect(group: &usvg::Group, k: f32, out: &mut Vec<Arm>) -> eros::Result<()> {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => collect(g, k, out)?,
            usvg::Node::Path(p) => {
                let Some(usvg::Paint::LinearGradient(g)) = p.fill().map(|f| f.paint()) else {
                    bail!("arm {} has no linear gradient", p.id())
                };
                let to_content = Transform::from_scale(k, k).pre_concat(p.abs_transform());
                let b = p
                    .data()
                    .clone()
                    .transform(to_content)
                    .context("transform arm")?
                    .bounds();
                let (dx, dy) = (b.x() + b.width() / 2.0 - C, b.y() + b.height() / 2.0 - C);
                let len = dx.hypot(dy);
                let stops = g.stops().iter().map(|s| {
                    let c = s.color();
                    (
                        s.offset().get(),
                        [c.red, c.green, c.blue],
                        s.opacity().get(),
                    )
                });
                out.push(Arm {
                    path: p.data().clone(),
                    rule: if p.fill().map(|f| f.rule()) == Some(usvg::FillRule::EvenOdd) {
                        FillRule::EvenOdd
                    } else {
                        FillRule::Winding
                    },
                    to_content,
                    gradient: (
                        Point::from_xy(g.x1(), g.y1()),
                        Point::from_xy(g.x2(), g.y2()),
                        stops.collect(),
                        g.transform(),
                    ),
                    dir: [dx / len, dy / len],
                    red: p.id().contains("red"),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn load_reveal(path: &FsPath, (w, h): (u32, u32)) -> eros::Result<Pixmap> {
    let img = image::open(path)
        .with_context(|| format!("open image {}", path.display()))?
        .to_rgba8();
    let k = (w as f32 / img.width() as f32).max(h as f32 / img.height() as f32);
    let (nw, nh) = (
        ((img.width() as f32 * k).ceil() as u32).max(w),
        ((img.height() as f32 * k).ceil() as u32).max(h),
    );
    let resized = imageops::resize(&img, nw, nh, imageops::FilterType::Lanczos3);
    let mut canvas = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 255]));
    imageops::overlay(
        &mut canvas,
        &resized,
        -i64::from((nw - w) / 2),
        -i64::from((nh - h) / 2),
    );
    Ok(Pixmap::from_vec(
        canvas.into_raw(),
        IntSize::from_wh(w, h).context("zero size")?,
    )
    .context("image to pixmap")?)
}

fn layer(pos: [f32; 2], rot: f32, scale: [f32; 2], anchor: f32) -> Transform {
    Transform::from_translate(pos[0], pos[1])
        .pre_concat(Transform::from_rotate(rot))
        .pre_concat(Transform::from_scale(scale[0] / 100.0, scale[1] / 100.0))
        .pre_concat(Transform::from_translate(-anchor, -anchor))
}

fn gradient(
    start: Point,
    end: Point,
    stops: &[(f32, [u8; 3], f32)],
    ts: Transform,
    alpha: f32,
) -> Option<Paint<'static>> {
    let stops = stops.iter().map(|&(at, [r, g, b], a)| {
        GradientStop::new(
            at,
            Color::from_rgba(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a * alpha,
            )
            .unwrap(),
        )
    });
    let shader = LinearGradient::new(start, end, stops.collect(), SpreadMode::Pad, ts)?;
    Some(Paint {
        shader,
        anti_alias: true,
        ..Paint::default()
    })
}

impl Scene {
    pub fn new(image: &FsPath, size: (u32, u32)) -> eros::Result<Self> {
        let svg = include_str!("../assets/logo.svg");
        let tree =
            usvg::Tree::from_str(svg, &usvg::Options::default()).context("parse logo svg")?;
        let mut arms = Vec::new();
        collect(tree.root(), 512.0 / tree.size().width(), &mut arms)?;
        // clockwise from the top
        let angle = |a: &Arm| (a.dir[1].atan2(a.dir[0]).to_degrees() + 90.0).rem_euclid(360.0);
        arms.sort_by(|a, b| angle(a).total_cmp(&angle(b)));
        let tl = Timeline::new(&arms.iter().map(|a| (a.dir, a.red)).collect::<Vec<_>>());
        Ok(Self {
            arms,
            tl,
            reveal: load_reveal(image, size)?,
            sc: size.0 as f32 / W,
            size,
        })
    }

    pub fn frame(&self, f: u32) -> Pixmap {
        let (t, tl, sc) = (f as f32, &self.tl, self.sc);
        let mut px = Pixmap::new(self.size.0, self.size.1).expect("canvas size");
        px.fill(Color::BLACK);
        let canvas = Transform::from_scale(sc, sc);
        let logo = canvas.pre_concat(layer(
            tl.logo_p.at(t),
            tl.logo_r.at(t)[0],
            [tl.logo_s.at(t)[0]; 2],
            C,
        ));

        // arms, bottom one first
        for (arm, tr) in self.arms.iter().zip(&tl.arms).rev() {
            let alpha = tr.o.at(t)[0] / 100.0;
            let [s] = tr.s.at(t);
            let ts = logo
                .pre_concat(layer(tr.p.at(t), tr.r.at(t)[0], [s; 2], C))
                .pre_concat(arm.to_content);
            let (a, b, stops, gt) = &arm.gradient;
            if let Some(paint) = gradient(*a, *b, stops, *gt, alpha).filter(|_| alpha > 0.001) {
                px.fill_path(&arm.path, &paint, arm.rule, ts, None);
            }
        }

        let alpha = tl.screen_o.at(t)[0] / 100.0;
        if alpha > 0.001 {
            let r = 1500.0 * tl.screen_s.at(t)[0] / 100.0 * sc;
            if let Some(circle) = PathBuilder::from_circle(CX * sc, FLOOR * sc, r) {
                let shader = Pattern::new(
                    self.reveal.as_ref(),
                    SpreadMode::Pad,
                    FilterQuality::Bilinear,
                    alpha,
                    Transform::identity(),
                );
                let paint = Paint {
                    shader,
                    anti_alias: true,
                    ..Paint::default()
                };
                px.fill_path(
                    &circle,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }

        let alpha = tl.drop_o.at(t)[0] / 100.0;
        if alpha > 0.001 {
            let mut d = PathBuilder::new();
            d.move_to(0.0, -44.0);
            for c in [
                [8.0, -22.0, 30.0, -8.0, 30.0, 12.0],
                [30.0, 29.0, 17.0, 42.0, 0.0, 42.0],
                [-17.0, 42.0, -30.0, 29.0, -30.0, 12.0],
                [-30.0, -8.0, -8.0, -22.0, 0.0, -44.0],
            ] {
                d.cubic_to(c[0], c[1], c[2], c[3], c[4], c[5]);
            }
            d.close();
            let stops: Stops = RED.iter().map(|&(at, rgb)| (at, rgb, 1.0)).collect();
            let ts = canvas
                .pre_concat(Transform::from_translate(
                    tl.drop_p.at(t)[0],
                    tl.drop_p.at(t)[1],
                ))
                .pre_concat(Transform::from_scale(
                    tl.drop_s.at(t)[0] / 100.0,
                    tl.drop_s.at(t)[1] / 100.0,
                ));
            if let (Some(path), Some(paint)) = (
                d.finish(),
                gradient(
                    Point::from_xy(-20.0, -44.0),
                    Point::from_xy(20.0, 42.0),
                    &stops,
                    Transform::identity(),
                    alpha,
                ),
            ) {
                px.fill_path(&path, &paint, FillRule::Winding, ts, None);
            }
        }

        let alpha = tl.ring_o.at(t)[0] / 100.0;
        if alpha > 0.001 {
            if let Some(ring) = PathBuilder::from_circle(0.0, 0.0, tl.ring_d.at(t)[0] / 2.0) {
                let mut paint = Paint::default();
                paint.set_color(
                    Color::from_rgba(
                        ACCENT[0] as f32 / 255.0,
                        ACCENT[1] as f32 / 255.0,
                        ACCENT[2] as f32 / 255.0,
                        alpha,
                    )
                    .unwrap(),
                );
                let stroke = Stroke {
                    width: 6.0,
                    ..Stroke::default()
                };
                px.stroke_path(
                    &ring,
                    &paint,
                    &stroke,
                    canvas.pre_concat(Transform::from_translate(CX, FLOOR)),
                    None,
                );
            }
        }
        px
    }
}
