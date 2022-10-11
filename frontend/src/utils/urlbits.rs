use reqwest::Url;
use serde_json::Value;

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
