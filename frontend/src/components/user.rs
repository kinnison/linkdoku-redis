//! Components related to users

use yew::prelude::*;

#[derive(Clone, Properties, Default, PartialEq, Eq)]
pub struct AvatarProps {
    pub name: String,
    pub email: Option<String>,
}

#[function_component(Avatar)]
pub fn user_avatar(props: &AvatarProps) -> Html {
    let namebits = props.name.split_whitespace().collect::<Vec<_>>();
    let initials = match namebits.len() {
        0 => "??".to_string(),
        1 => format!("{}", namebits[0].chars().next().unwrap()),
        _ => {
            let first = namebits[0];
            let last = namebits[namebits.len() - 1];
            format!(
                "{}{}",
                first.chars().next().unwrap(),
                last.chars().next().unwrap()
            )
        }
    };

    if let Some(email) = props.email.as_deref() {
        // Email provided, so try and do a gravatar
        let email = email.trim();
        let hash = format!("{:x}", md5::compute(email.as_bytes()));
        html! {
            <figure class={"image is-48x48"}>
                <img class={"is-rounded"} src={format!("https://www.gravatar.com/avatar/{}", hash)} />
            </figure>
        }
    } else {
        // No email, so we need to get some initials together
        html! {
            <figure class={"image is-48x48 has-text-centered"}>
                <span class={"is-lowercase subtitle is-4"}>{initials}</span>
            </figure>
        }
    }
}
