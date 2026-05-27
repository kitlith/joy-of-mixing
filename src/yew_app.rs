use std::{borrow::Cow, collections::HashMap};

use joy_of_mixing::Color;
use web_sys::{FormData, HtmlFormElement, console};
use yew::prelude::*;

use crate::{find_color_mix, mix_dep_order};

struct State {
    mixes: HashMap<Color, Vec<Color>>,
    final_color: Option<Color>,
    target_color: Color,
    iterations: usize,
    max_mix_count: usize,
}

#[component]
pub fn App() -> Html {
    let state = use_state(|| State {
        mixes: HashMap::new(),
        final_color: None,
        target_color: 0x9b9b9b.into(),
        iterations: 10,
        max_mix_count: 8,
    });

    let onsubmit = {
        let state = state.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let form = event.target_unchecked_into::<HtmlFormElement>();
            let data = FormData::new_with_form(&form).expect("unable to get form data");

            // TODO: Error handling
            let target_color = data.get("color").as_string().unwrap();
            let target_color: Color =
                u32::from_str_radix(target_color.strip_prefix('#').unwrap_or(&target_color), 16)
                    .unwrap()
                    .into();

            console::debug_1(&data.get("use_all_colors").js_typeof());
            let iterations =
                usize::from_str_radix(&data.get("iterations").as_string().unwrap(), 10).unwrap();
            let use_all_colors = data.get("use_all_colors").is_truthy();
            let max_mix_count =
                usize::from_str_radix(&data.get("max_mix_count").as_string().unwrap(), 10).unwrap();

            if let Some(mixes) = find_color_mix(
                Color::BASIC_COLORS.iter().map(|(c, _)| c.clone()),
                target_color.clone(),
                iterations,
                use_all_colors,
                max_mix_count,
            ) {
                // TODO: this is duplicated between yew and cli!
                let final_color = if mixes.contains_key(&target_color) {
                    target_color.clone()
                } else {
                    let found = mixes
                        .keys()
                        .min_by(|a, b| {
                            a.distance_squared(&target_color)
                                .total_cmp(&b.distance_squared(&target_color))
                        })
                        .expect("find_color_mix returned empty mix map???")
                        .clone();

                    //println!("Unable to find target color. Closest color: {found:?}");

                    found
                };

                state.set(State {
                    mixes,
                    final_color: Some(final_color),
                    target_color,
                    iterations,
                    max_mix_count,
                });
            } else {
                state.set(State {
                    mixes: HashMap::new(),
                    final_color: None,
                    target_color,
                    iterations,
                    max_mix_count,
                });
            }
        })
    };

    let mut display = Vec::new();
    let mut color_names: HashMap<Color, Cow<str>> = Color::BASIC_COLORS
        .into_iter()
        .map(|(c, l)| (c, l.into()))
        .collect();

    if let Some(ref final_color) = state.final_color {
        color_names.insert(final_color.clone(), "result".into());
        mix_dep_order(
            final_color.clone(),
            &state.mixes,
            &mut color_names,
            &mut display,
        );
    }

    html! {
        <div>
            <p>{"Color mixing calculator for Joy of Painting"}</p>
            <p><a href="https://github.com/kitlith/joy-of-mixing">{"Source Code"}</a></p>
            <form {onsubmit}>
                <label for="color" title="Color in hex notation (i.e. #9b9b9b)">{"Target Color: "}</label>
                <input type="text" name="color" pattern="#?[0-9a-fA-F]{6}" value={format!("{}", state.target_color)} required=true />
                <br/>
                <label for="number" title="Number of colors to create before giving up">{"Iteration Count: "}</label>
                <input type="number" name="iterations" value={format!("{}", state.iterations)} required=true />
                <br/>
                <label for="checkbox" title="Uses all colors instead of just the colors in the bounding tetrahedron">{"Use All Colors: "}</label>
                <input type="checkbox" name="use_all_colors" />
                <br/>
                <label for="max_mix_count" title="Maxiumum number of color parts to use for each new color">{"Max Mix Count: "}</label>
                <input type="number" name="max_mix_count" value={format!("{}", state.max_mix_count)} required=true />
                <br/>
                <input type="submit" value="Get recipe" required=true />
            </form>
            <ul>
            for mix_display in display {
                <li>
                    {&color_names[&mix_display.color]} {" ("} {&mix_display.color} {")"}
                    <ul>
                    for color in mix_display.mix {
                        <li>{&*color_names[&color.0]}{" x "}{color.1}</li>
                    }
                    </ul>
                </li>
            }
            </ul>
        </div>
    }
}
