use std::{borrow::Cow, collections::HashMap};

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
    let mut color_names: HashMap<Color, Cow<str>> = Color::BASIC_COLORS
        .into_iter()
        .map(|(c, l)| (c, l.into()))
        .collect();
    let basic_colors = Color::BASIC_COLORS.map(|(c, _)| c);

    let args: Args = argh::from_env();

    let target_color = args.target_color;
    println!("Target Color: {:?}", target_color);

    let color_mixes = find_color_mix(
        &basic_colors,
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

    print_mix_deps(final_color, &color_mixes, &mut color_names, &mut 0);
}

fn print_mix_deps(
    target: Color,
    mixes: &HashMap<Color, Vec<Color>>,
    names: &mut HashMap<Color, Cow<str>>,
    counter: &mut usize,
) {
    let mix = if let Some(mix) = mixes.get(&target) {
        mix
    } else {
        return;
    };

    let mut coalesced: Vec<_> = mix
        .iter()
        .cloned()
        .map(|c| (c, 1))
        .coalesce(|p, c| {
            (p.0 == c.0)
                .then_some((p.0.clone(), p.1 + c.1))
                .ok_or((p, c))
        })
        .inspect(|(c, _)| print_mix_deps(c.clone(), mixes, names, counter))
        .collect();

    coalesced.sort_unstable_by_key(|(_, c)| *c);

    if !names.contains_key(&target) {
        let label = format!("tmp_{}", *counter);
        *counter += 1;
        names.insert(target.clone(), label.into());
    }

    let label = names.get(&target).unwrap();

    println!(
        "{label}({target}) => [\n    {}\n]",
        coalesced
            .iter()
            .format_with(",\n    ", |(color, count), fmt| fmt(&format_args!(
                "{} x {count}",
                names[color]
            )))
    );
}

fn find_color_mix(
    base_colors: &[Color],
    target_color: Color,
    iterations: usize,
    use_all_colors: bool,
    max_mix_count: usize,
) -> Option<HashMap<Color, Vec<Color>>> {
    let mut color_bounds = base_colors
        .iter()
        .cloned()
        .array_combinations::<4>()
        .map(ColorBounds)
        .filter(|b| b.contains(&target_color))
        // TODO: find another heuristic?
        //  maybe (signed distance from tetrahrdron, distance from the center of the tetrahedron)?
        .min_by_key(|b| b.volume_6())?;

    let mut mix_map = HashMap::new();
    let all_colors: Option<Vec<_>> = use_all_colors.then(|| base_colors.to_vec());

    for _ in 0..iterations {
        // place the bounding vertex farthest from the target color in the last slot
        color_bounds.0.sort_unstable_by(|a, b| {
            a.distance_squared(&target_color)
                .total_cmp(&b.distance_squared(&target_color))
        });

        let used_colors = all_colors
            .as_ref()
            .map(<Vec<_>>::as_slice)
            .unwrap_or(color_bounds.0.as_slice());

        let (new_color, new_color_mix) =
            target_color.find_closest_mix(&used_colors, max_mix_count, |try_color| {
                if try_color == target_color {
                    return true; // we found the color we want!
                }

                // we're going to replace the farthest vertex with the result, so ensure
                // that the target color is still within bounds
                let mut new_bounds = color_bounds.clone();
                new_bounds.0[3] = try_color.clone();
                new_bounds.contains(&target_color)
            });

        mix_map.insert(new_color.clone(), new_color_mix);
        color_bounds.0[3] = new_color.clone();

        if new_color == target_color {
            break;
        }
    }

    Some(mix_map)
}
