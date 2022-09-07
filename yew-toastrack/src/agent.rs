use std::collections::HashSet;

use yew_agent::{Agent, AgentLink, Context, HandlerId};

use crate::Toast;

pub(crate) struct ToastVan {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for ToastVan {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Toast;
    type Output = Toast;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        for sub in &self.subscribers {
            self.link.respond(*sub, msg.clone())
        }
    }

    fn connected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.subscribers.insert(id);
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.subscribers.remove(&id);
        }
    }
}
