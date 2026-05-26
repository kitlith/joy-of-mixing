use std::collections::HashMap;

use itertools::Itertools;

use joy_of_mixing::{Color, ColorBounds};

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

fn main() {
    let mut color_names: HashMap<Color, &str> = Color::BASIC_COLORS.into_iter().collect();

    let args: Args = argh::from_env();

    let target_color = args.target_color;
    println!("Target Color: {:?}", target_color);

    let mut color_bounds = color_names
        .keys()
        .cloned()
        .array_combinations::<4>()
        .map(ColorBounds)
        .filter(|b| b.contains(&target_color))
        // TODO: find another heuristic?
        //  maybe (signed distance from tetrahrdron, distance from the center of the tetrahedron)?
        .min_by_key(|b| b.volume_6())
        .expect("Target color was not inside gamut!");

    println!(
        "Starting Tetrahedron: {}, volume: {}",
        color_bounds.0.iter().map(|c| color_names[c]).format(", "),
        color_bounds.volume_6(),
    );

    for color_idx in 0..args.iterations {
        // find the color part of the bounding tetrahedron that is furthest from the target color
        color_bounds.0.sort_unstable_by(|a, b| {
            a.distance_squared(&target_color)
                .total_cmp(&b.distance_squared(&target_color))
        });

        let all_colors: Option<Vec<_>> = args
            .use_all_colors
            .then(|| color_names.keys().cloned().collect());

        let used_colors = all_colors
            .as_ref()
            .map(<Vec<_>>::as_slice)
            .unwrap_or(color_bounds.0.as_slice());

        let (new_color, new_color_mix) =
            target_color.find_closest_mix(&used_colors, args.max_mix_count, |try_color| {
                let mut new_bounds = color_bounds.clone();
                new_bounds.0[3] = try_color.clone();
                try_color == target_color || new_bounds.contains(&target_color)
            });

        if new_color == target_color {
            println!("Final Mix({:?}): => [", new_color);
            for col in new_color_mix {
                println!("    {}", color_names[col]);
            }
            println!("]");
            return;
        }

        let new_label: &str = format!("color_{}", color_idx).leak();

        color_names.insert(new_color.clone(), new_label);

        println!(
            "{new_label}({new_color:?}) replacing {}: => [",
            color_names[&color_bounds.0[3]]
        );
        for col in new_color_mix {
            println!("    {}", color_names[col]);
        }
        println!("]");

        color_bounds.0[3] = new_color;
    }

    println!(
        "Failed to find target color in {} iterations.",
        args.iterations
    );
}
