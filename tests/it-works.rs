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

struct Test {
    cb: MsgCallbacks<Self>,
}

impl Component for Test {
    type Properties = ();
    type Message = Msg;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            cb: ctx.link().into(),
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let cb1: Callback<()> = self.cb.on_click();
        let cb2: Callback<()> = self.cb.on_click();
        assert_eq!(cb1, cb2);

        let cb1: Callback<(InputEvent, KeyboardEvent)> = self.cb.on_input();
        let cb2: Callback<(InputEvent, KeyboardEvent)> = self.cb.on_input();
        assert_eq!(cb1, cb2);

        let cb1: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_key_press(0, "foo".to_string());
        let cb2: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_key_press(0, "foo".to_string());
        let cb3: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_key_press(1, "foo".to_string());
        assert_eq!(cb1, cb2);
        assert_ne!(cb1, cb3);

        let cb1: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_stuff(0, "foo".to_string());
        let cb2: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_stuff(0, "foo".to_string());
        let cb3: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_stuff(1, "foo".to_string());
        assert_eq!(cb1, cb2);
        assert_ne!(cb1, cb3);

        let cb1: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_other_stuff();
        let cb2: Callback<(KeyboardEvent, InputEvent)> = self.cb.on_other_stuff();
        assert_eq!(cb1, cb2);

        html! {}
    }
}

#[test]
fn run_tests() {
    futures::executor::block_on(async {
        let renderer = yew::ServerRenderer::<Test>::new();
        let _ = renderer.render().await;
    });
}
