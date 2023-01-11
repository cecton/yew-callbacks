use yew::prelude::*;
use yew_callbacks::Callbacks;

#[derive(Callbacks)]
pub enum Msg {
    OnClick,
    OnInput(InputEvent, KeyboardEvent),
    OnKeyPress(#[curry] usize, KeyboardEvent, InputEvent, #[curry] String),
    OnStuff {
        #[curry]
        index: usize,
        kb_event: KeyboardEvent,
        input_event: InputEvent,
        #[curry]
        key: String,
    },
    OnOtherStuff {
        kb_event: KeyboardEvent,
        input_event: InputEvent,
    },
}

fn main() {}
