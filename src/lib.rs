use std::rc::Rc;

use dominator::{events, Dom};
use futures_signals::signal::{Mutable, Signal, SignalExt};
use handler::{Extract, Handler};
use path::{Path, PathMatch};
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::{Element, EventTarget};

pub use path::Params;

mod handler;
mod path;

pub type RouteHandler = Rc<dyn Fn(&ReadOnlyRouter, &PathMatch) -> Option<Dom>>;

#[derive(Default)]
pub struct Router {
    path: Mutable<Path>,
    path_offset: usize,
    routes: Vec<(Path, RouteHandler)>,
    default: Option<String>,
    on_change: Option<Box<dyn Fn(&Path)>>,
}

impl Router {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_browser_url() -> Self {
        Self {
            on_change: Some(Box::new(|p| {
                web_sys::window()
                    .unwrap()
                    .history()
                    .unwrap()
                    .push_state_with_url(&JsValue::NULL, "", Some(&p.to_string()))
                    .unwrap_throw();
            })),
            path: Mutable::new(Path::new(
                &web_sys::window()
                    .and_then(|w| w.location().pathname().ok())
                    .unwrap_or_default(),
            )),
            ..Default::default()
        }
    }

    pub fn readonly(&self) -> ReadOnlyRouter {
        ReadOnlyRouter {
            path: self.path.clone(),
            path_offset: self.path_offset,
            default: self.default.clone(),
        }
    }

    pub fn add_route<H, Args>(&mut self, route: &str, handler: H)
    where
        H: Handler<Args> + 'static,
        Args: Extract,
    {
        self.routes.push((
            Path::new(route),
            Rc::new(move |router, mtch| handler.call(Args::extract(router, mtch))),
        ));
    }

    pub fn set_default(&mut self, path: &str) {
        self.default.replace(path.to_string());
    }

    pub fn signal_active(&self, path: &str) -> impl Signal<Item = bool> {
        self.path.signal_cloned().map({
            let route = Path::new(path);
            let offset = self.path_offset;

            move |path| route.matches(&path, offset).is_some()
        })
    }

    pub fn mount(self) -> impl Signal<Item = Option<Dom>> {
        struct InternalMatch((PathMatch, RouteHandler));

        impl PartialEq for InternalMatch {
            fn eq(&self, other: &Self) -> bool {
                self.0 .0 == other.0 .0
            }
        }

        let router = self.readonly();

        self.path
            .signal_ref({
                move |path| {
                    if let Some(f) = &self.on_change {
                        f(path);
                    }

                    for (route, handler) in &self.routes {
                        if let Some(mtch) = route.matches(path, self.path_offset) {
                            return Some(InternalMatch((mtch, handler.clone())));
                        }
                    }

                    None
                }
            })
            .dedupe_map({
                move |mtch| {
                    if let Some(InternalMatch((m, h))) = mtch {
                        return (h)(&router, m);
                    }

                    if let Some(path) = &router.default {
                        router.goto(path);
                    }

                    None
                }
            })
    }
}

#[derive(Clone)]
pub struct ReadOnlyRouter {
    path: Mutable<Path>,
    path_offset: usize,
    default: Option<String>,
}

impl ReadOnlyRouter {
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
                        .take(self.path_offset)
                        .collect(),
                ) + path
            }
        };

        self.path.set_neq(new);
    }

    pub fn signal_active(&self, path: &str) -> impl Signal<Item = bool> {
        self.path.signal_cloned().map({
            let route = Path::new(path);
            let offset = self.path_offset;

            move |path| route.matches(&path, offset).is_some()
        })
    }
}

impl From<&Router> for ReadOnlyRouter {
    fn from(value: &Router) -> Self {
        value.readonly()
    }
}

impl From<&ReadOnlyRouter> for ReadOnlyRouter {
    fn from(value: &ReadOnlyRouter) -> Self {
        value.clone()
    }
}

pub fn active_router_link<B, R>(
    router: R,
    target: &str,
) -> impl FnOnce(dominator::DomBuilder<B>) -> dominator::DomBuilder<B>
where
    B: AsRef<EventTarget> + AsRef<Element>,
    R: Into<ReadOnlyRouter>,
{
    let router: ReadOnlyRouter = router.into();
    let target = target.to_string();
    move |dom| {
        dom.class_signal("active", router.signal_active(&target))
            .event(move |_: events::Click| router.goto(&target))
    }
}

pub fn router_link<B, R>(
    router: R,
    target: &str,
) -> impl FnOnce(dominator::DomBuilder<B>) -> dominator::DomBuilder<B>
where
    B: AsRef<EventTarget> + AsRef<Element>,
    R: Into<ReadOnlyRouter>,
{
    let router: ReadOnlyRouter = router.into();
    let target = target.to_string();
    move |dom| dom.event(move |_: events::Click| router.goto(&target))
}
