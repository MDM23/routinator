use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dominator::{events, html, Dom, DomBuilder};
use futures_signals::signal::{Mutable, Signal, SignalExt};
use path::Path;
use route::Route;

mod handler;
mod path;
mod route;

pub use route::ToRoute;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::{Element, EventTarget};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Params(HashMap<String, String>);

impl Params {
    pub fn get(&self, k: &str) -> Option<&String> {
        self.0.get(k)
    }

    fn insert(&mut self, k: &str, v: &str) {
        self.0.insert(k.to_owned(), v.to_owned());
    }
}

pub struct RouteContext {
    route: Route,
    router: Router,
    params: Params,
}

impl PartialEq for RouteContext {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route && self.params == other.params
    }
}

pub struct Router {
    path: Mutable<Path>,
    path_offset: usize,
    matched_segments: Rc<RefCell<usize>>,
    on_change: Option<Box<dyn Fn(&Path)>>,
    is_cloned: bool,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            path_offset: self.path_offset.clone(),
            matched_segments: self.matched_segments.clone(),
            on_change: None,
            is_cloned: true,
        }
    }
}

impl Router {
    pub fn new(initial_path: &str) -> Self {
        Router {
            path: Mutable::new(Path::new(initial_path)),
            path_offset: 0,
            matched_segments: Rc::new(RefCell::new(0)),
            on_change: None,
            is_cloned: false,
        }
    }

    pub fn with_browser_url() -> Self {
        Router {
            path: Mutable::new(Path::new(&get_current_url())),
            path_offset: 0,
            matched_segments: Rc::new(RefCell::new(0)),
            on_change: Some(Box::new(|p| set_current_url(&p.to_string()))),
            is_cloned: false,
        }
    }

    pub fn goto(&self, target: &str) {
        // Only treat the target as absolute path if explicitly asked to do so
        let target = if !target.starts_with(['/', '.']) {
            String::from("./") + target
        } else {
            target.to_string()
        };

        let new = match Path::new(&target) {
            path @ Path::Absolute(_) => path,
            path @ Path::Relative(_) => {
                Path::Absolute(
                    self.path
                        .get_cloned()
                        .into_segments()
                        .take(self.path_offset + self.matched_segments.borrow().saturating_sub(1))
                        .collect(),
                ) + path
            }
        };

        self.path.set_neq(new);
    }

    pub fn nest(&self) -> Router {
        Router {
            path: self.path.clone(),
            path_offset: *self.matched_segments.borrow(),
            matched_segments: Rc::new(RefCell::new(0)),
            on_change: None,
            is_cloned: false,
        }
    }

    pub fn signal_active(&self, path: &str) -> impl Signal<Item = bool> {
        self.path.signal_cloned().map({
            let route = Path::new(path);
            let offset = self.path_offset;

            move |path| route.matches(&path, offset)
        })
    }

    pub fn link_active<B>(&self, target: &str) -> impl FnOnce(DomBuilder<B>) -> DomBuilder<B>
    where
        B: AsRef<EventTarget> + AsRef<Element>,
    {
        let target = target.to_string();
        let router = self.clone();
        move |dom| {
            dom.class_signal("active", router.signal_active(&target))
                .event(move |_: events::Click| router.goto(&target))
        }
    }

    pub fn link<B>(&self, target: &str) -> impl FnOnce(DomBuilder<B>) -> DomBuilder<B>
    where
        B: AsRef<EventTarget> + AsRef<Element>,
    {
        let target = target.to_string();
        let router = self.clone();
        move |dom| dom.event(move |_: events::Click| router.goto(&target))
    }

    pub fn mount<const N: usize>(self, routes: [Route; N]) -> Dom {
        html!("router", {
            .child_signal(self.mount_signal(routes))
        })
    }

    pub fn mount_signal<const N: usize>(
        self,
        routes: [Route; N],
    ) -> impl Signal<Item = Option<Dom>> + 'static {
        if self.is_cloned {
            panic!("Cannot mount cloned router! Please use nest() or create a new instance.");
        }

        let router = self.clone();

        self.path
            .signal_ref(move |path| {
                for r in &routes {
                    if let Some((m, p)) = r.matches(path, self.path_offset) {
                        router.matched_segments.replace(m);

                        if let Some(f) = &self.on_change {
                            f(path);
                        }

                        return Some(RouteContext {
                            route: r.clone(),
                            router: router.clone(),
                            params: p,
                        });
                    }
                }

                None
            })
            .dedupe_map(|ctx| ctx.as_ref().and_then(|ctx| (ctx.route.handler)(ctx)))
    }
}

fn get_current_url() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default()
}

fn set_current_url(path: &str) {
    web_sys::window()
        .unwrap()
        .history()
        .unwrap()
        .push_state_with_url(&JsValue::NULL, "", Some(&path.to_string()))
        .unwrap_throw();
}
