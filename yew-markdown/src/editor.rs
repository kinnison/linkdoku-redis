//! Editor for markdown - this is a tabbed control using Bulma CSS
//! classes to control the tabs etc.  We show a text area of the name
//! provided to us, along with a preview pane.
//!
//! For now, text is fixed, but later we will support changing that too

use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_bulma_tabs::{TabContent, Tabbed};
use yew_hooks::use_debounce_state;

use crate::render::MarkdownRender;

#[derive(Properties, PartialEq, Eq)]
pub struct MarkdownEditorProps {
    pub name: String,
    pub initial: String,
}

#[function_component(MarkdownEditor)]
pub fn markdown_editor(props: &MarkdownEditorProps) -> Html {
    let markdown = use_state(|| props.initial.clone());

    let editor = use_node_ref();

    let onchange = {
        let setter = markdown.clone();
        let editor = editor.clone();
        Callback::from(move |_| {
            let editor: HtmlTextAreaElement = editor.cast().unwrap();
            let value = editor.value();
            setter.set(value);
        })
    };

    let oninput = {
        let setter = markdown.clone();
        let editor = editor.clone();
        Callback::from(move |_| {
            let editor: HtmlTextAreaElement = editor.cast().unwrap();
            let value = editor.value();
            setter.set(value);
        })
    };

    html! {
        <Tabbed default={"Write"}>
            <TabContent title={"Write"}>
                <textarea ref={editor} onchange={onchange} oninput={oninput} class={"textarea is-family-code"} name={props.name.clone()} value={(*markdown).clone()} />
            </TabContent>
            <TabContent title={"Preview"}>
                <MarkdownRender markdown={(*markdown).clone()} />
            </TabContent>
        </Tabbed>
    }
}
