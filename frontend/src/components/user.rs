//! Components related to users

use yew::prelude::*;

use crate::components::login::{LoginButton, LoginStatus, LogoutButton};

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
            <figure class={"image is-32x32"}>
                <img class={"is-rounded"} style={"max-height: inherit;"} src={format!("https://www.gravatar.com/avatar/{}", hash)} />
            </figure>
        }
    } else {
        // No email, so we need to get some initials together
        html! {
            <figure class={"image is-32x32 has-text-centered"}>
                <span class={"is-lowercase subtitle is-4"}>{initials}</span>
            </figure>
        }
    }
}

#[function_component(UserMenuNavbarItem)]
pub fn user_menu_button() -> Html {
    match use_context::<LoginStatus>().expect("Unable to retrieve login status") {
        LoginStatus::Unknown => html! {},
        LoginStatus::LoggedOut => html! {
            <div class={"navbar-item"}>
                <div class={"buttons"}>
                    <LoginButton />
                </div>
            </div>
        },
        LoginStatus::LoggedIn { name, email } => html! {
            <div class={"navbar-item has-dropdown is-hoverable"}>
                <a class={"navbar-link"}>
                    <Avatar name={name} email={email} />
                </a>

                <div class={"navbar-dropdown"}>
                    <div class={"navbar-item"}>
                        <div class={"buttons"}>
                            <LogoutButton />
                        </div>
                    </div>
                </div>
            </div>
        },
    }
}
