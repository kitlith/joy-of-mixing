use std::{borrow::Cow, collections::HashMap};

use itertools::Itertools;

use joy_of_mixing::Color;

use crate::{MixDisplayInfo, find_color_mix, mix_dep_order};

/// Find a mix of colors that produces a target color
#[derive(argh::FromArgs)]
struct Args {
    /// target color given in RGB or ARGB notation
    #[argh(positional)]
    target_color: Color,
    /// number of colors to create before giving up
    #[argh(option, default = "10")]
    iterations: usize,
    /// uses all colors instead of just the colors in the bounding tetrahedron
    /// note: turning this on will make things a little bit slower
    #[argh(switch)]
    use_all_colors: bool,
    /// maxiumum number of color parts to use for each new color
    /// note: turning this higher will make things slower
    #[argh(option, default = "8")]
    max_mix_count: usize,
}

pub fn cli_main() {
    let mut color_names: HashMap<Color, Cow<str>> = Color::BASIC_COLORS
        .into_iter()
        .map(|(c, l)| (c, l.into()))
        .collect();
    let basic_colors = Color::BASIC_COLORS.iter().map(|(c, _)| c.clone());

    let args: Args = argh::from_env();

    let target_color = args.target_color;
    println!("Target Color: {:?}", target_color);

    let color_mixes = find_color_mix(
        basic_colors,
        target_color.clone(),
        args.iterations,
        args.use_all_colors,
        args.max_mix_count,
    )
    .expect("target color out of gamut!");

    let final_color = if color_mixes.contains_key(&target_color) {
        target_color
    } else {
        let found = color_mixes
            .keys()
            .min_by(|a, b| {
                a.distance_squared(&target_color)
                    .total_cmp(&b.distance_squared(&target_color))
            })
            .expect("find_color_mix returned empty mix map???")
            .clone();

        println!("Unable to find target color. Closest color: {found:?}");

        found
    };

    color_names.insert(final_color.clone(), "result".into());

    let mut display = Vec::new();
    mix_dep_order(final_color, &color_mixes, &mut color_names, &mut display);

    for MixDisplayInfo { color, mix } in display {
        let label = &color_names[&color];
        println!(
            "{label}({color}) => [\n    {}\n]",
            mix
                .iter()
                .format_with(",\n    ", |(color, count), fmt| fmt(&format_args!(
                    "{} x {count}",
                    color_names[color]
                )))
        );
    }
}
