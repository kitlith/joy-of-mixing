use std::{borrow::Cow, collections::HashMap};

use itertools::Itertools;

use joy_of_mixing::{Color, ColorBounds};

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod cli;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn main() {
    cli::cli_main();
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod yew_app;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn main() {
    // TODO: generate initial content as SSG and then hydrate?
    yew::Renderer::<yew_app::App>::new().render();
}

fn find_color_mix(
    base_colors: impl Iterator<Item = Color> + Clone,
    target_color: Color,
    iterations: usize,
    use_all_colors: bool,
    max_mix_count: usize,
) -> Option<HashMap<Color, Vec<Color>>> {
    let mut color_bounds = base_colors.clone()
        .array_combinations::<4>()
        .map(ColorBounds)
        .filter(|b| b.contains(&target_color))
        // TODO: find another heuristic?
        //  maybe (signed distance from tetrahrdron, distance from the center of the tetrahedron)?
        .min_by_key(|b| b.volume_6())?;

    let mut mix_map = HashMap::new();
    let mut all_colors: Option<Vec<_>> = use_all_colors.then(|| base_colors.collect());

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

        if new_color == target_color {
            break;
        }

        if let Some(all) = all_colors.as_mut() {
            all.push(new_color.clone())
        }

        color_bounds.0[3] = new_color;
    }

    Some(mix_map)
}

struct MixDisplayInfo {
    color: Color,
    mix: Vec<(Color, usize)>,
}

fn mix_dep_order(
    target: Color,
    mixes: &HashMap<Color, Vec<Color>>,
    names: &mut HashMap<Color, Cow<str>>,
    out: &mut Vec<MixDisplayInfo>,
) {
    let mix = if let Some(mix) = mixes.get(&target) {
        mix
    } else {
        return;
    };

    // this color has already been processed
    if out.iter().find(|m| m.color == target).is_some() {
        return;
    }

    let mut coalesced: Vec<_> = mix
        .iter()
        .cloned()
        .map(|c| (c, 1))
        .coalesce(|p, c| {
            (p.0 == c.0)
                .then_some((p.0.clone(), p.1 + c.1))
                .ok_or((p, c))
        })
        .inspect(|(c, _)| mix_dep_order(c.clone(), mixes, names, out))
        .collect();

    coalesced.sort_unstable_by_key(|(_, c)| *c);

    if !names.contains_key(&target) {
        let label = format!("tmp_{}", out.len());
        names.insert(target.clone(), label.into());
    }

    out.push(MixDisplayInfo { color: target, mix: coalesced });
}
