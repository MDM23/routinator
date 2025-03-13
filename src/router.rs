use dominator::{events, Dom, DomBuilder, EventOptions};
use futures_signals::signal::{Mutable, Signal, SignalExt};
use gloo::{events::EventListener, utils::window};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use web_sys::{wasm_bindgen::JsValue, Element, EventTarget};

use crate::path::{Path, Route};

#[inline]
fn current_path() -> Path {
    window()
        .location()
        .pathname()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default()
}

pub trait MaybeDom {
    fn into_option_dom(self) -> Option<Dom>;
}

impl MaybeDom for Dom {
    fn into_option_dom(self) -> Option<Dom> {
        Some(self)
    }
}

impl MaybeDom for Option<Dom> {
    fn into_option_dom(self) -> Option<Dom> {
        self
    }
}

pub trait Handler<A> {
    fn execute(&self, router: &Router) -> Option<Dom>;
}

impl<F, R> Handler<()> for F
where
    F: Fn() -> R,
    R: MaybeDom,
{
    fn execute(&self, _router: &Router) -> Option<Dom> {
        (self)().into_option_dom()
    }
}

impl<F, R> Handler<&Router> for F
where
    F: Fn(Router) -> R,
    R: MaybeDom,
{
    fn execute(&self, router: &Router) -> Option<Dom> {
        (self)(Router {
            root: router.root.clone(),
            parent: router
                .parent
                .clone()
                .merge_opt(router.context.borrow().clone()),
            context: Default::default(),
            routes: Default::default(),
            default_path: Default::default(),
            popstate: None,
        })
        .into_option_dom()
    }
}

#[derive(Debug, Clone, Default)]
struct Context {
    path: Path,
    params: HashMap<String, String>,
}

impl Context {
    pub fn merge_opt(self, rhs: Option<Self>) -> Self {
        let Some(rhs) = rhs else {
            return self;
        };

        Self {
            path: self.path + rhs.path,
            params: self
                .params
                .into_iter()
                .chain(rhs.params.into_iter())
                .collect(),
        }
    }
}

pub struct Router {
    root: Mutable<Path>,
    parent: Context,
    context: Rc<RefCell<Option<Context>>>,
    routes: Vec<(Route, Box<dyn Fn(&Router) -> Option<Dom>>)>,
    default_path: Option<Path>,
    #[allow(dead_code)]
    popstate: Option<EventListener>,
}

impl Router {
    pub fn root() -> Self {
        let root = Mutable::new(current_path());

        Self {
            root: root.clone(),
            parent: Default::default(),
            context: Default::default(),
            routes: Default::default(),
            default_path: Default::default(),
            popstate: Some(EventListener::new(&window(), "popstate", move |_| {
                root.set_neq(current_path());
            })),
        }
    }

    pub fn route<A>(mut self, path: &str, handler: impl Handler<A> + 'static) -> Self {
        self.routes.push((
            path.parse().unwrap(),
            Box::new(move |router| handler.execute(router)),
        ));

        self
    }

    pub fn default(mut self, path: &str) -> Self {
        self.default_path = Some(path.parse().unwrap());
        self
    }

    pub fn handle(&self) -> RouterHandle {
        RouterHandle {
            root: self.root.clone(),
            parent: self.parent.clone(),
            current: self.context.clone(),
        }
    }

    pub fn mount(mut self) -> impl Signal<Item = Option<Dom>> {
        let routes: Vec<Route> = self.routes.iter().map(|(r, _)| r.clone()).collect();
        let default_path = self.default_path.take();

        self.root
            .signal_cloned()
            .map({
                let handle = self.handle();
                move |p| {
                    for (i, r) in routes.iter().enumerate() {
                        let test = p.skip(handle.parent.path.len());
                        if let Some((p, par)) = r.match_path(&test) {
                            handle.current.replace(Some(Context {
                                path: handle.parent.path.clone() + p,
                                params: par,
                            }));
                            return Some(i);
                        }
                    }

                    handle.current.replace(None);

                    if let Some(p) = &default_path {
                        handle.replace(&p.to_string());
                    }

                    None
                }
            })
            .dedupe()
            .map(move |i| i.and_then(|i| (self.routes.get(i).unwrap().1)(&self)))
    }

    pub fn link<B>(&self, path: &str) -> impl FnOnce(DomBuilder<B>) -> DomBuilder<B> + '_
    where
        B: AsRef<EventTarget> + AsRef<Element>,
    {
        let path = path.to_string();
        let handle = self.handle();

        // TODO: Only set href for actual <a> nodes?
        move |dom| {
            dom.attr("href", &handle.link_target(&path).to_string())
                .class_signal("routinator-active", self.signal_active(&path))
                .event_with_options(&EventOptions::preventable(), move |e: events::Click| {
                    if !e.ctrl_key() && !e.shift_key() {
                        e.prevent_default();
                        handle.goto(&path);
                    }
                })
        }
    }

    pub fn param(&self, key: &str) -> Option<String> {
        self.context
            .borrow()
            .as_ref()
            .and_then(|ctx| ctx.params.get(key))
            .or_else(|| self.parent.params.get(key))
            .cloned()
    }

    pub fn signal_active(&self, path: &str) -> impl Signal<Item = bool> {
        let handle = self.handle();
        let route: Route = path.parse().unwrap();

        self.root.signal_ref(move |p| {
            route
                .match_path(&p.skip(handle.parent.path.len()))
                .is_some()
        })
    }
}

pub struct RouterHandle {
    root: Mutable<Path>,
    parent: Context,
    current: Rc<RefCell<Option<Context>>>,
}

impl RouterHandle {
    pub fn goto(&self, target: &str) {
        let target = self.link_target(target);

        // This does not trigger a popstate event, so we need to update the URL
        // afterwards to keep everything in sync.
        window()
            .history()
            .unwrap()
            .push_state_with_url(&JsValue::null(), "", Some(&target.to_string()))
            .unwrap();

        self.root.set_neq(target);
    }

    pub fn replace(&self, target: &str) {
        let target = self.link_target(target);

        // This does not trigger a popstate event, so we need to update the URL
        // afterwards to keep everything in sync.
        window()
            .history()
            .unwrap()
            .replace_state_with_url(&JsValue::null(), "", Some(&target.to_string()))
            .unwrap();

        self.root.set_neq(target);
    }

    pub fn param(&self, key: &str) -> Option<String> {
        self.current
            .borrow()
            .as_ref()
            .and_then(|ctx| ctx.params.get(key))
            .or_else(|| self.parent.params.get(key))
            .cloned()
    }

    fn link_target(&self, target: &str) -> Path {
        self.parent.path.clone() + target.parse().unwrap()
    }
}
