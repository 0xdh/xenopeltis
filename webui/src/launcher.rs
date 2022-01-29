use yew::prelude::*;
use yew::virtual_dom::VChild;

pub enum Msg {
    AddOne,
    Connect,
}

pub struct Launcher {
    value: i64,
}

#[derive(Properties, PartialEq)]
pub struct Props {}

impl Component for Launcher {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
            Msg::Connect => true,
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <button onclick={link.callback(|_| Msg::Connect)}>{ "Connect" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}
