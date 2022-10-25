use linkdoku_common::{PuzzleData, PuzzleState};
use reqwest::Url;
use serde_json::Value;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_markdown::render::MarkdownRender;
use yew_markdown::xform::{TransformRequest, TransformResponse};

pub fn extract_fpuzzles_data(input: &str) -> Option<Value> {
    fn maybe_decode_fpuzzles(input: &str) -> Option<Value> {
        //gloo::console::log!(format!("Attempting to decode lzstr: {}", input));
        let decompressed = lz_str::decompress_from_base64(input)?;
        //gloo::console::log!(format!("Attempting to load that as a String"));
        let decompressed = String::from_utf16(&decompressed).ok()?;
        //gloo::console::log!(format!("Attempting to parse as JSON: {}", decompressed));
        serde_json::from_str(&decompressed).ok()
    }

    fn find_arg(url: &Url, key_to_find: &'static str) -> Option<String> {
        //gloo::console::log!(format!(
        //    "Attempting to find {} in {}",
        //    key_to_find,
        //    url.query().unwrap_or("{noquery}")
        //));
        url.query_pairs()
            .find(|(key, _)| key == key_to_find)
            .map(|(_, value)| value)
            .map(|load| {
                gloo::console::log!(format!("Found it: {}", load));
                load.to_string()
            })
    }

    //gloo::console::log!(format!("Attempt to extract fpuzzles from: {}", input));

    if let Ok(url) = Url::parse(input) {
        //gloo::console::log!(format!("OK, it's a URL, hostname is {:?}", url.host_str()));
        // there are two URL forms that we understand, the f-puzzles load form, and the sudokupad form
        if let Some(host) = url.host_str() {
            if let Some(data) = match host {
                "f-puzzles.com" => find_arg(&url, "load"),
                _ if host.ends_with("sudokupad.app")
                    || host.ends_with("app.crackingthecryptic.com") =>
                {
                    find_arg(&url, "puzzleid")
                        .and_then(|s| s.strip_prefix("fpuzzles").map(String::from))
                        .or_else(|| {
                            //gloo::console::log!(
                            //    "Oh well, trying query string without parsing it..."
                            //);
                            url.query()
                                .and_then(|s| s.strip_prefix("fpuzzles").map(String::from))
                        })
                        .or_else(|| {
                            //gloo::console::log!(format!(
                            //    "Oh well, trying the path: {}",
                            //    url.path()
                            //));
                            url.path().strip_prefix("/fpuzzles").map(String::from)
                        })
                }
                _ if host.ends_with("sudokulab.net") => find_arg(&url, "fpuzzle"),
                _ => None,
            } {
                // Unfortunately sometimes we end up with plusses in our encoded data, and that is needed
                // so reestablish those just in case
                let data = data.replace(' ', "+");
                //gloo::console::log!(format!("Found something to try and decode: {}", data));
                if let Some(value) = maybe_decode_fpuzzles(&data) {
                    return Some(value);
                }
            }
        }
    }
    //gloo::console::log!("Sadly, not managed a decode yet, try the whole string");
    // Not parseable as a recognisable URL, so try and just treat it as fpuzzles data raw
    maybe_decode_fpuzzles(input)
}

pub fn encode_fpuzzles_data(value: &Value) -> String {
    let json_data = serde_json::to_string(value).expect("Odd, JSON encoding failed?");
    lz_str::compress_to_base64(json_data.as_str())
}

pub fn grid_svg_url(value: &Value) -> String {
    format!(
        "https://api.sudokupad.com/thumbnail/fpuzzles{}_512x512.svg",
        encode_fpuzzles_data(value)
    )
}

pub fn trivially_text(node: &Html, target: &str) -> bool {
    if let VNode::VList(l) = node {
        if l.len() == 1 {
            if let VNode::VList(l) = &l[0] {
                if l.len() == 1 {
                    if let VNode::VText(text) = &l[0] {
                        if target == &*text.text {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn transform_markdown(grid: &PuzzleState, req: TransformRequest) -> TransformResponse {
    fn error(msg: String) -> TransformResponse {
        Some(html! {
            <strong><em>{msg}</em></strong>
        })
    }

    match req {
        TransformRequest::Link {
            url,
            title,
            content,
        } => {
            if let Some(maybe_idx) = url.strip_prefix("url-") {
                match maybe_idx.parse::<usize>() {
                    Ok(num) if num > 0 => {
                        if let PuzzleData::URLs(urls) = &grid.data {
                            if let Some(ue) = urls.get(num - 1) {
                                let content = if trivially_text(&content, &url) {
                                    html! {{ue.title.clone()}}
                                } else {
                                    content
                                };
                                let title = if title == url {
                                    ue.title.clone()
                                } else {
                                    title
                                };
                                Some(html! {
                                    <a href={ue.url.clone()} title={title.clone()}>{content}</a>
                                })
                            } else {
                                error(format!("URL index out of range: {}", num))
                            }
                        } else {
                            error(format!("Use of {} in non-URLs form puzzle state", url))
                        }
                    }
                    _ => error(format!("Bad number in `url-{}`", maybe_idx)),
                }
            } else if let Some(maybe_idx) = url.strip_prefix("puzzle-") {
                match maybe_idx.parse::<usize>() {
                    Ok(num) if num > 0 => {
                        if let PuzzleData::Pack(urls) = &grid.data {
                            if let Some(ue) = urls.get(num - 1) {
                                let content = if trivially_text(&content, &url) {
                                    html! {
                                        <em>{"TODO: Magic puzzle link content"}</em>
                                    }
                                } else {
                                    content
                                };
                                Some(html! {
                                    <span>{"This would be a puzzle link to "} {ue}{". "} {content}</span>
                                })
                            } else {
                                error(format!("Puzzle index out of range: {}", num))
                            }
                        } else {
                            error(format!("Use of {} in non-pack form puzzle state", url))
                        }
                    }
                    _ => error(format!("Bad number in `puzzle-{}`", maybe_idx)),
                }
            } else {
                match url.as_str() {
                    "grid" | "rules" | "fpuzzles" | "sudokupad" | "beta-sudokupad"
                    | "sudokupad-beta" => {
                        // Must have an fpuzzles dataset
                        if let PuzzleData::FPuzzles(grid) = &grid.data {
                            match url.as_str() {
                                "grid" => error(
                                    "Use of [grid] as a non-image link.  Did you mean `![grid]` instead?"
                                        .to_string(),
                                ),
                                "rules" => {
                                    let rules = grid
                                        .as_object()
                                        .and_then(|o| o.get("ruleset"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("No rules available in data");
                                    Some(html! {
                                        <MarkdownRender markdown={rules.to_string()} />
                                    })
                                }
                                "fpuzzles" | "sudokupad" | "beta-sudokupad" | "sudokupad-beta" => {
                                    let data_str = encode_fpuzzles_data(grid);
                                    gloo::console::log!(format!("Content: {:?}", content));
                                    gloo::console::log!(format!("Checker: {:?}", html! {{url.clone()}}));
                                    let content = if trivially_text(&content, &url) {
                                        html! {
                                            {match url.as_str() {
                                                "fpuzzles" => "Play this on F-Puzzles",
                                                "sudokupad" => "Play this on Sudokupad",
                                                "beta-sudokupad" | "sudokupad-beta" => "Play this on Sudokupad (beta)",
                                                _ => unreachable!(),
                                            }}
                                        }
                                    } else {
                                        content
                                    };
                                    let link = match url.as_str() {
                                        "fpuzzles" => {
                                            format!("http://f-puzzles.com/?load={}", data_str)
                                        }
                                        "sudokupad" => {
                                            format!("https://sudokupad.app/fpuzzles{}", data_str)
                                        }
                                        "beta-sudokupad" | "sudokupad-beta" => format!(
                                            "https://beta.sudokupad.app/fpuzzles{}",
                                            data_str,
                                        ),
                                        _ => unreachable!(),
                                    };
                                    Some(html! {
                                        <a href={link}>{content}</a>
                                    })
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            error(format!(
                                "Use of `{}` in a non-fpuzzles form puzzle state",
                                url
                            ))
                        }
                    }
                    _ => error(format!("Unknown special link: `{}`", url)),
                }
            }
        }
        TransformRequest::Image { url, .. } => {
            if url == "grid" {
                if let PuzzleData::FPuzzles(grid) = &grid.data {
                    Some(html! {
                        <img src={grid_svg_url(grid)} style={"width: 50vh; height: 50vh;"} />
                    })
                } else {
                    error("Use of ![grid] in a non-fpuzzles puzzle state".to_string())
                }
            } else {
                None
            }
        }
    }
}
