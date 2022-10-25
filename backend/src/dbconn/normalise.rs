use super::Database;
use super::DatabaseResult;

use redis::Cmd;

/// Reserved short names which are not permitted for convenience / clarity reasons
const RESERVED_SHORT_NAMES: &[&str] = &[
    "api", "-", "linkdoku", "r", "p", "role", "puzzle", "create", "delete", "rename",
];

/// Normalise a short name name, and ensure it is unique.
/// Note: this is no guarantee of uniqueness by the time you get to the server later, but it's
/// a good way to ensure nothing unusual happens.
pub async fn unique_short_name(
    database: &mut Database,
    short_name: &str,
    group: &str,
) -> DatabaseResult<String> {
    // Step one is to take the lower-cased ascii version of short_name
    let mut short_name = short_name.to_ascii_lowercase();
    // Next we replace any spaces with underscores
    short_name = short_name.replace(' ', "_");
    // Next, we remove anything which isn't `-`, `_`, `.`, a letter, or a digit
    short_name.retain(|c| "abcdefghijklmnopqrstuvwxyz0123456789-_.".contains(c));
    // Now we take the role name and if it's a reserved word we add an underscore afterwards
    if RESERVED_SHORT_NAMES.iter().any(|&s| s == short_name) {
        short_name.push('_');
    }
    // Finally we set a counter at zero, and we try and find a unique role name...
    let mut full_short_name = short_name.clone();
    let mut counter = 0;
    let group_key = format!("{}:byname", group);
    loop {
        let found: bool = Cmd::hexists(&group_key, &full_short_name)
            .query_async(&mut database.conn)
            .await?;
        if !found {
            break Ok(full_short_name);
        }
        full_short_name = format!("{}_{}", short_name, counter);
        counter += 1;
    }
}
