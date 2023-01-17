![Rust](https://github.com/cecton/yew-callbacks/actions/workflows/rust.yml/badge.svg)
[![Latest Version](https://img.shields.io/crates/v/yew-callbacks.svg)](https://crates.io/crates/yew-callbacks)
![Rust 1.46+](https://img.shields.io/badge/rust-1.60%2B-orange.svg)
![License](https://img.shields.io/crates/l/yew-callbacks)
[![Docs.rs](https://docs.rs/yew-callbacks/badge.svg)](https://docs.rs/yew-callbacks)
[![LOC](https://tokei.rs/b1/github/cecton/yew-callbacks)](https://github.com/cecton/yew-callbacks)
[![Dependency Status](https://deps.rs/repo/github/cecton/yew-callbacks/status.svg)](https://deps.rs/repo/github/cecton/yew-callbacks)

yew-callbacks
=============

<!-- cargo-rdme start -->

Yet another crate nobody asked for.

This crate provides a derive macro `Callbacks` that can be used on Yew enum messages to help
managing callbacks.

### But why

Callbacks in Yew's components are easy to create but hard to manage. To avoid duplication you
should create them preemptively in the `create()` method of your component, store them in the
state of your component, then pass clones to the children. Unfortunately this creates a lot of
bloat.

To address this, `yew-callbacks` provides a macro that will automatically create some kind of
cache for your callbacks. You create this cache once in the `create()` method of your component
and then you can use the methods to get your callbacks easily.

#### Example

```rust
use yew::prelude::*;
use yew_callbacks::Callbacks;

#[derive(Debug, Callbacks)]
enum Msg {
    OnClick(MouseEvent),
}

#[derive(Debug)]
struct App {
    cb: MsgCallbacks<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            cb: ctx.link().into(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <button onclick={self.cb.on_click()}>
                { "Hello World!" }
            </button>
        }
    }
}
```

### Why care

Not perf.

Your children components will be updated if their properties changed. If you do
`onclick={ctx.link().callback(Msg::OnClick)` then the child component will think there is an
update every time the parent component updates. This is because doing
`ctx.link().callback(Msg::OnClick)` creates a new callback every time.

### Handling multiple child components

This crate also allows currying the arguments of your callback.

#### Example

```rust
use yew::prelude::*;
use yew_callbacks::Callbacks;

#[derive(Debug, Callbacks)]
enum Msg {
    OnClick(#[curry] usize, MouseEvent),
}

#[derive(Debug)]
struct App {
    games: Vec<AttrValue>,
    cb: MsgCallbacks<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            games: vec![
                "Freedom Planet 2".into(),
                "Asterigos: Curse of the Stars".into(),
                "Fran Bow".into(),
                "Cats in Time".into(),
                "Ittle Dew 2+".into(),
                "Inscryption".into(),
            ],
            cb: ctx.link().into(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self
            .games
            .iter()
            .enumerate()
            .map(|(i, game)| html! {
                <button onclick={self.cb.on_click(i)}>
                    { format!("You should try {game}") }
                </button>
            })
            .collect()
    }
}
```

<!-- cargo-rdme end -->
