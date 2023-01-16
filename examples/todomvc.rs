// example todomvc mostly copied from Yew's examples

use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};
use web_sys::HtmlInputElement as InputElement;
use yew::prelude::*;
use yew_callbacks::Callbacks;

const KEY: &str = "yew.todomvc.self";

#[derive(Debug, Callbacks)]
enum Msg {
    Add(KeyboardEvent),
    OnBlur(#[curry] usize, FocusEvent),
    OnKeyPress(#[curry] usize, KeyboardEvent),
    Edit(#[curry] usize, InputElement),
    Remove(#[curry] usize, MouseEvent),
    SetFilter(#[curry] Filter, MouseEvent),
    ToggleAll(MouseEvent),
    ToggleEdit(#[curry] usize, MouseEvent),
    Toggle(#[curry] usize, MouseEvent),
    ClearCompleted(MouseEvent),
    Focus(MouseEvent),
}

#[derive(Debug)]
struct App {
    entries: Vec<Entry>,
    filter: Filter,
    edit_value: String,
    focus_ref: NodeRef,
    cb: MsgCallbacks<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            entries: LocalStorage::get(KEY).unwrap_or_else(|_| Vec::new()),
            filter: Filter::All,
            edit_value: "".into(),
            focus_ref: NodeRef::default(),
            cb: ctx.link().into(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Add(event) => {
                if event.key() == "Enter" {
                    let input: InputElement = event.target_unchecked_into();
                    let description = input.value();
                    input.set_value("");
                    if !description.is_empty() {
                        let entry = Entry {
                            description: description.trim().to_string(),
                            completed: false,
                            editing: false,
                        };
                        self.entries.push(entry);
                    }
                }
            }
            Msg::OnBlur(idx, event) => {
                ctx.link()
                    .send_message(Msg::Edit(idx, event.target_unchecked_into()));
            }
            Msg::OnKeyPress(idx, event) => {
                if event.key() == "Enter" {
                    ctx.link()
                        .send_message(Msg::Edit(idx, event.target_unchecked_into()));
                }
            }
            Msg::Edit(idx, input) => {
                let edit_value = input.value();
                input.set_value("");
                self.complete_edit(idx, edit_value.trim().to_string());
                self.edit_value = "".to_string();
            }
            Msg::Remove(idx, _event) => {
                self.remove(idx);
            }
            Msg::SetFilter(filter, _event) => {
                self.filter = filter;
            }
            Msg::ToggleEdit(idx, _event) => {
                let entry = self
                    .entries
                    .iter()
                    .filter(|e| self.filter.fits(e))
                    .nth(idx)
                    .unwrap();
                self.edit_value = entry.description.clone();
                self.clear_all_edit();
                self.toggle_edit(idx);
            }
            Msg::ToggleAll(_event) => {
                let status = !self.is_all_completed();
                for entry in &mut self.entries {
                    if self.filter.fits(entry) {
                        entry.completed = status;
                    }
                }
            }
            Msg::Toggle(idx, _event) => {
                let filter = self.filter;
                let entry = self
                    .entries
                    .iter_mut()
                    .filter(|e| filter.fits(e))
                    .nth(idx)
                    .unwrap();
                entry.completed = !entry.completed;
            }
            Msg::ClearCompleted(_event) => {
                let entries = self
                    .entries
                    .drain(..)
                    .filter(|e| Filter::Active.fits(e))
                    .collect();
                self.entries = entries;
            }
            Msg::Focus(_event) => {
                if let Some(input) = self.focus_ref.cast::<InputElement>() {
                    input.focus().unwrap();
                }
            }
        }
        LocalStorage::set(KEY, &self.entries).expect("failed to set");
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="todomvc-wrapper">
                <section class="todoapp">
                    <header class="header">
                        <h1>{ "todos" }</h1>
                        { self.view_input() }
                    </header>
                    <section class={classes!("main", self.entries.is_empty().then_some("hidden"))}>
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            checked={self.is_all_completed()}
                            onclick={self.cb.toggle_all()}
                        />
                        <label for="toggle-all" />
                        <ul class="todo-list">
                            {
                                for self
                                    .entries
                                    .iter()
                                    .filter(|e| self.filter.fits(e))
                                    .enumerate()
                                    .map(|(idx, e)| self.view_entry(idx, e))
                            }
                        </ul>
                    </section>
                    <footer
                        class={classes!("footer", self.entries.is_empty().then_some("hidden"))}
                    >
                        <span class="todo-count">
                            <strong>{ self.total() }</strong>
                            { " item(s) left" }
                        </span>
                        <ul class="filters">
                            { for Filter::iter().map(|flt| self.view_filter(flt)) }
                        </ul>
                        <button class="clear-completed" onclick={self.cb.clear_completed()}>
                            { format!("Clear completed ({})", self.total_completed()) }
                        </button>
                    </footer>
                </section>
                <footer class="info">
                    <p>{ "Double-click to edit a todo" }</p>
                    <p>{ "Written by " }<a href="https://github.com/DenisKolodin/" target="_blank">{ "Denis Kolodin" }</a></p>
                    <p>{ "Part of " }<a href="http://todomvc.com/" target="_blank">{ "TodoMVC" }</a></p>
                </footer>
            </div>
        }
    }
}

impl App {
    fn view_filter(&self, filter: Filter) -> Html {
        let cls = if self.filter == filter {
            "selected"
        } else {
            "not-selected"
        };
        html! {
            <li>
                <a class={cls}
                   href={filter.as_href()}
                   onclick={self.cb.set_filter(filter)}
                >
                    { filter }
                </a>
            </li>
        }
    }

    fn view_input(&self) -> Html {
        html! {
            // You can use standard Rust comments. One line:
            // <li></li>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                onkeypress={self.cb.add()}
            />
            /* Or multiline:
            <ul>
                <li></li>
            </ul>
            */
        }
    }

    fn view_entry(&self, idx: usize, entry: &Entry) -> Html {
        let mut class = Classes::from("todo");
        if entry.editing {
            class.push(" editing");
        }
        if entry.completed {
            class.push(" completed");
        }
        html! {
            <li {class}>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked={entry.completed}
                        onclick={self.cb.toggle(idx)}
                    />
                    <label ondblclick={self.cb.toggle_edit(idx)}>{ &entry.description }</label>
                    <button class="destroy" onclick={self.cb.remove(idx)} />
                </div>
                { self.view_entry_edit_input(idx, entry) }
            </li>
        }
    }

    fn view_entry_edit_input(&self, idx: usize, entry: &Entry) -> Html {
        if entry.editing {
            html! {
                <input
                    class="edit"
                    type="text"
                    ref={self.focus_ref.clone()}
                    value={self.edit_value.clone()}
                    onmouseover={self.cb.focus()}
                    onblur={self.cb.on_blur(idx)}
                    onkeypress={self.cb.on_key_press(idx)}
                />
            }
        } else {
            html! { <input type="hidden" /> }
        }
    }

    fn total(&self) -> usize {
        self.entries.len()
    }

    fn total_completed(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| Filter::Completed.fits(e))
            .count()
    }

    fn is_all_completed(&self) -> bool {
        let mut filtered_iter = self
            .entries
            .iter()
            .filter(|e| self.filter.fits(e))
            .peekable();

        if filtered_iter.peek().is_none() {
            return false;
        }

        filtered_iter.all(|e| e.completed)
    }

    fn toggle_edit(&mut self, idx: usize) {
        let filter = self.filter;
        let entry = self
            .entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .nth(idx)
            .unwrap();
        entry.editing = !entry.editing;
    }

    fn clear_all_edit(&mut self) {
        for entry in &mut self.entries {
            entry.editing = false;
        }
    }

    fn complete_edit(&mut self, idx: usize, val: String) {
        if val.is_empty() {
            self.remove(idx);
        } else {
            let filter = self.filter;
            let entry = self
                .entries
                .iter_mut()
                .filter(|e| filter.fits(e))
                .nth(idx)
                .unwrap();
            entry.description = val;
            entry.editing = !entry.editing;
        }
    }

    fn remove(&mut self, idx: usize) {
        let idx = {
            let entries = self
                .entries
                .iter()
                .enumerate()
                .filter(|&(_, e)| self.filter.fits(e))
                .collect::<Vec<_>>();
            let &(idx, _) = entries.get(idx).unwrap();
            idx
        };
        self.entries.remove(idx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    description: String,
    completed: bool,
    editing: bool,
}

#[derive(Clone, Copy, Debug, EnumIter, Display, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    fn fits(&self, entry: &Entry) -> bool {
        match *self {
            Filter::All => true,
            Filter::Active => !entry.completed,
            Filter::Completed => entry.completed,
        }
    }

    fn as_href(&self) -> &'static str {
        match self {
            Filter::All => "#/",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
        }
    }
}

#[xtask_wasm::run_example(index = r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Yew • TodoMVC</title>
    <link
      rel="stylesheet"
      href="https://cdn.jsdelivr.net/npm/todomvc-common@1.0.5/base.css"
    />
    <link
      rel="stylesheet"
      href="https://cdn.jsdelivr.net/npm/todomvc-app-css@2.3.0/index.css"
    />
    <script type="module">
    import init from "/app.js";
    init(new URL('app.wasm', import.meta.url));
    </script>
  </head>
  <body></body>
</html>"#)]
fn main() {
    yew::Renderer::<App>::new().render();
}
