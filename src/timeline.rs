use std::collections::BTreeMap;

use bezier_easing::bezier_easing;

pub const W: f32 = 1920.0;
pub const H: f32 = 1080.0;
pub const CX: f32 = W / 2.0;
pub const CY: f32 = 470.0;
pub const FLOOR: f32 = H;
pub const C: f32 = 256.0; // centre of the logo in its own 512 px space
pub const FIT: f32 = 60.0; // logo size, percent of that space
pub const K: f32 = 1.05; // drop size

const FAR: f32 = 300.0;
const SPIN: f32 = 70.0;
const STAGGER: f32 = 3.0;
const TRAVEL: f32 = 26.0;
const TURN: (f32, f32) = (46.0, 91.0); // the 120 degree turn in the loop
const GAP: f32 = 3.0;
const PULL: f32 = 13.0;

pub const INTRO_END: u32 = 46;
pub const LOOP_END: u32 = INTRO_END + 72;
const ABSORB: u32 = LOOP_END + 8;
const LAST_END: u32 = ABSORB + 15 + PULL as u32;
const GROW: (f32, f32) = (ABSORB as f32 + 5.0, LAST_END as f32 - 4.0);
const WOBBLE: u32 = LAST_END + 2;
const T_FALL: u32 = WOBBLE + 10;
const T_HIT: u32 = T_FALL + 26;
pub const FRAMES: u32 = T_HIT + 46;

#[derive(Clone, Copy, PartialEq)]
pub enum Ease {
    Out,
    InOut,
    Glide,
    In,
    Swing,
    Strike,
    Fall,
    Playful,
    Hold,
}

impl Ease {
    fn bezier(self) -> [f32; 4] {
        match self {
            Self::Out => [0.23, 1.0, 0.32, 1.0],
            Self::InOut => [0.77, 0.0, 0.175, 1.0],
            Self::Glide => [0.32, 0.72, 0.0, 1.0],
            Self::In => [0.55, 0.0, 1.0, 0.45],
            Self::Swing => [0.37, 0.0, 0.63, 1.0],
            Self::Strike => [0.61, 1.0, 0.88, 1.0],
            Self::Fall => [1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0 / 3.0],
            Self::Playful => [0.34, 1.56, 0.64, 1.0],
            Self::Hold => unreachable!("hold has no curve"),
        }
    }
}

pub struct Track<const N: usize>(Vec<(f32, [f32; N], Ease)>);

impl<const N: usize> Track<N> {
    pub fn at(&self, t: f32) -> [f32; N] {
        let k = &self.0;
        let Some(i) = k.iter().rposition(|key| key.0 <= t) else {
            return k[0].1;
        };
        let Some(next) = k.get(i + 1) else {
            return k[i].1;
        };
        let (t0, from, ease) = k[i];
        if ease == Ease::Hold {
            return from;
        }
        let [x1, y1, x2, y2] = ease.bezier();
        let p = bezier_easing(x1, y1, x2, y2)
            .expect("valid curve")
            .sample((t - t0) / (next.0 - t0));
        std::array::from_fn(|j| from[j] + (next.1[j] - from[j]) * p)
    }
}

macro_rules! track {
    ($(($t:expr, $v:expr $(, $e:ident)?)),+ $(,)?) => {
        Track(vec![$(($t as f32, $v, track!(@ease $($e)?))),+])
    };
    (@ease) => { Ease::Out };
    (@ease $e:ident) => { Ease::$e };
}

pub struct ArmTracks {
    pub p: Track<2>,
    pub r: Track<1>,
    pub s: Track<1>,
    pub o: Track<1>,
}

impl ArmTracks {
    fn new(i: usize, dir: [f32; 2], red: bool) -> Self {
        let far = [C + dir[0] * FAR, C + dir[1] * FAR];
        let sign = if red { 1.0 } else { -1.0 };
        let t = STAGGER * i as f32;
        let b = ABSORB as f32 + GAP * i as f32;
        let e = b + PULL;
        Self {
            p: track![(t, far, Playful), (t + TRAVEL, [C, C])],
            r: track![
                (t, [sign * SPIN], Playful),
                (t + TRAVEL, [0.0], Out),
                (b, [0.0], InOut),
                (e, [-sign * 35.0])
            ],
            s: track![(b, [100.0], InOut), (e, [10.0])],
            o: track![
                (t, [0.0], Out),
                (t + 9.0, [100.0], Out),
                (e - 4.0, [100.0], In),
                (e - 1.0, [0.0])
            ],
        }
    }
}

pub struct Timeline {
    pub arms: Vec<ArmTracks>,
    pub logo_p: Track<2>,
    pub logo_s: Track<1>,
    pub logo_r: Track<1>,
    pub drop_p: Track<2>,
    pub drop_s: Track<2>,
    pub drop_o: Track<1>,
    pub ring_d: Track<1>,
    pub ring_o: Track<1>,
    pub screen_s: Track<1>,
    pub screen_o: Track<1>,
}

impl Timeline {
    pub fn new(arms: &[([f32; 2], bool)]) -> Self {
        let base = |t: f32| 0.35 + 0.65 * ((t - GROW.0) / (GROW.1 - GROW.0)).clamp(0.0, 1.0);
        let ends: Vec<u32> = (0..arms.len() as u32)
            .map(|i| ABSORB + 3 * i + PULL as u32)
            .collect();

        let mut keys: BTreeMap<u32, ([f32; 2], Ease)> = BTreeMap::new();
        keys.insert(
            GROW.0 as u32 - 4,
            ([base(GROW.0) * 100.0 * K; 2], Ease::Out),
        );
        for &e in &ends {
            keys.insert(e, ([base(e as f32) * 110.0 * K; 2], Ease::InOut));
            keys.insert(e + 2, ([base(e as f32 + 2.0) * 100.0 * K; 2], Ease::InOut));
        }
        keys.extend([
            (WOBBLE, ([100.0 * K; 2], Ease::InOut)),
            (WOBBLE + 2, ([106.0 * K, 94.0 * K], Ease::Swing)),
            (WOBBLE + 4, ([96.0 * K, 105.0 * K], Ease::Swing)),
            (WOBBLE + 6, ([102.0 * K, 99.0 * K], Ease::Swing)),
            (WOBBLE + 8, ([100.0 * K; 2], Ease::InOut)),
            (T_FALL, ([92.0 * K, 112.0 * K], Ease::InOut)),
            (T_HIT, ([88.0 * K, 118.0 * K], Ease::Strike)),
            (T_HIT + 5, ([150.0 * K, 36.0 * K], Ease::Out)),
        ]);
        let hit = T_HIT as f32;
        let (fall, bottom) = (42.0 * K, FLOOR);

        Self {
            arms: arms
                .iter()
                .enumerate()
                .map(|(i, &(dir, red))| ArmTracks::new(i, dir, red))
                .collect(),
            logo_p: track![
                (33, [CX, CY], Swing),
                (36, [CX + 8.0, CY - 4.0], Swing),
                (39, [CX - 7.0, CY + 4.0], Swing),
                (42, [CX + 4.0, CY - 2.0], Swing),
                (45, [CX, CY])
            ],
            logo_s: track![(LOOP_END, [FIT], InOut), (ABSORB, [FIT * 0.92])],
            logo_r: track![
                (TURN.0, [0.0], InOut),
                (TURN.1, [120.0], InOut),
                (LOOP_END, [120.0], InOut),
                (LAST_END, [240.0])
            ],
            drop_p: track![
                (T_FALL, [CX, CY], Fall),
                (hit, [CX, bottom - fall * 1.18], Out),
                (hit + 5.0, [CX, bottom - fall * 0.36])
            ],
            drop_s: Track(
                keys.into_iter()
                    .map(|(t, (v, e))| (t as f32, v, e))
                    .collect(),
            ),
            drop_o: track![
                (GROW.0 - 4.0, [0.0], Out),
                (GROW.0 + 1.0, [100.0], Out),
                (hit + 3.0, [100.0], In),
                (hit + 7.0, [0.0])
            ],
            ring_d: track![(hit, [12.0], Out), (hit + 30.0, [1500.0])],
            ring_o: track![
                (hit, [0.0], Hold),
                (hit + 1.0, [100.0], Out),
                (hit + 30.0, [0.0])
            ],
            screen_s: track![(hit + 2.0, [3.0], Glide), (hit + 40.0, [100.0])],
            screen_o: track![
                (hit + 1.0, [0.0], Hold),
                (hit + 2.0, [100.0], Out),
                (FRAMES, [100.0])
            ],
        }
    }
}
