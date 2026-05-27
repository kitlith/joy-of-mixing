use argh::FromArgValue;
use itertools::Itertools;
use nalgebra::{Matrix4, Point3, distance_squared};
use std::{
    f32,
    fmt::{Debug, Display},
};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct Color(Point3<u8>);

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Self {
        Self(value.into())
    }
}

impl FromArgValue for Color {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        u32::from_str_radix(value, 16)
            .map(<_>::into)
            .map_err(|e| format!("Error parsing color: {}", e))
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = (self.0[0] as u32) << 16 | (self.0[1] as u32) << 8 | (self.0[2] as u32) << 0;
        write!(f, "Color(0x{:06x})", color)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = (self.0[0] as u32) << 16 | (self.0[1] as u32) << 8 | (self.0[2] as u32) << 0;
        write!(f, "#{:06x}", color)
    }
}

impl Color {
    // since const traits are unstable
    const fn from_u32(value: u32) -> Self {
        let bytes = value.to_be_bytes();
        Self(Point3::new(bytes[1], bytes[2], bytes[3]))
    }
    // TODO: perceptual distance
    pub fn distance_squared(&self, other: &Color) -> f32 {
        distance_squared(&self.0.map(f32::from), &other.0.map(f32::from))
    }

    pub fn find_closest_mix<'a>(
        &self,
        colors: &'a [Color],
        max_mix_colors: usize,
        mut filter: impl FnMut(Color) -> bool,
    ) -> (Color, Vec<Color>) {
        (1..=max_mix_colors)
            .flat_map(move |num| colors.iter().cloned().combinations_with_replacement(num))
            .map(|mix| {
                (
                    mix.iter()
                        .fold(MixedColor::default(), |mut mix, col| {
                            mix.mix(col);
                            mix
                        })
                        .result(),
                    mix,
                )
            })
            .filter(|(col, _)| filter(col.clone()))
            .min_by(|(a, _), (b, _)| {
                a.distance_squared(self)
                    .total_cmp(&b.distance_squared(self))
            })
            .unwrap()
    }

    pub const BASIC_COLORS: [(Color, &str); 16] = [
        (Color::from_u32(0xFF1D1D21), "Black"),
        (Color::from_u32(0xFFB02E26), "Red"),
        (Color::from_u32(0xFF5E7C16), "Green"),
        (Color::from_u32(0xFF835432), "Brown"),
        (Color::from_u32(0xFF3C44AA), "Blue"),
        (Color::from_u32(0xFF8932B8), "Purple"),
        (Color::from_u32(0xFF169C9C), "Aqua"),
        (Color::from_u32(0xFF9D9D97), "Grey"),
        (Color::from_u32(0xFF474F52), "Dark Grey"),
        (Color::from_u32(0xFFF38BAA), "Pink"),
        (Color::from_u32(0xFF80C71F), "Lime"),
        (Color::from_u32(0xFFFED83D), "Yellow"),
        (Color::from_u32(0xFF3AB3DA), "Light Blue"),
        (Color::from_u32(0xFFC74EBD), "Magenta"),
        (Color::from_u32(0xFFF9801D), "Orange"),
        (Color::from_u32(0xFFF9FFFE), "White"),
    ];
}

#[derive(Default, Clone)]
pub struct MixedColor {
    total: Point3<i32>,
    max: i32,
    count: i32,
}

impl MixedColor {
    pub fn mix(&mut self, color: &Color) -> &mut Self {
        // let color = color.0.map(::from);
        self.total += color.0.coords.map(i32::from);
        self.max += color.0.coords.iter().fold(0, |a, b| a.max(*b)) as i32;
        self.count += 1;
        self
    }

    pub fn result(&self) -> Color {
        let avg = self.total / self.count;
        let avg_max = self.max / self.count;
        let max_avg = avg.iter().fold(0, |a, b| a.max(*b));

        // NOTE: This is where joy of painting color mixing diverges from armor color mixing.
        // Armor color mixing uses a float instead of an int for the gain.
        // While it is possible to mix colors such that gain does something
        // (i.e. pure red + pure green + pure blue)
        // with the base colors provided by the game the highest I've seen so far
        // is around 1.75, which rounds down to 1 since everything here is using integers.
        let gain: i32 = if max_avg == 0 { 0 } else { avg_max / max_avg };

        Color((avg * gain).map(|c| c as u8))
    }

    // pub fn gain_factor(&self) -> f32 {
    //     let avg = self.total / self.count;
    //     let avg_max = self.max / self.count;

    //     let max_avg = avg.iter().fold(0, |a, b| a.max(*b));
    //     if max_avg == 0 {
    //         0.
    //     } else {
    //         avg_max as f32 / max_avg as f32
    //     }
    // }
}

#[derive(Clone)]
pub struct ColorBounds(pub [Color; 4]);

impl ColorBounds {
    /// The volume of the bounding tetrahedron, multiplied by 6
    pub fn volume_6(&self) -> i64 {
        let v = self.0.clone().map(|vn| vn.0.map(i64::from));
        (v[0] - v[3])
            .dot(&(v[1] - v[3]).cross(&(v[2] - v[3])))
            .abs()
    }

    pub fn contains(&self, color: &Color) -> bool {
        let tet_matrix = Matrix4::from_columns(
            &self
                .0
                .clone()
                .map(|vn| vn.0.coords.map(f32::from).insert_row(3, 1.)),
        );
        let p = color.0.coords.map(f32::from).insert_row(3, 1.);

        let check_sign = tet_matrix.determinant().signum();

        for i in 0..4 {
            let mut test_matrix = tet_matrix.clone();
            test_matrix.set_column(i, &p);
            if test_matrix.determinant().signum() != check_sign {
                return false;
            }
        }

        true
    }
}
