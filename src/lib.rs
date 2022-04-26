use std::borrow::Borrow;
use std::rc::Rc;
use std::str::FromStr;

use dominator::{html, Dom};
use futures_signals::signal::{LocalBoxSignal, Mutable, Signal, SignalExt};
use handler::{Extract, Handler, HandlerContext};
use path::{Path, Route};

pub use path::Params;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

mod handler;
mod path;

thread_local! {
    static URL: Mutable<Path> = Mutable::new(
        Path::from_str(
            &web_sys::window()
                .and_then(|w| w.location().pathname().ok())
                .unwrap_or_default(),
        )
        .unwrap_or_default(),
    );
}

pub fn goto(path: &str) {
    URL.with(|u| {
        let path = Path::from_str(path).unwrap();

        let target = if path.is_relative() {
            u.borrow().get_cloned() + path
        } else {
            path
        };

        web_sys::window()
            .unwrap()
            .history()
            .unwrap()
            .push_state_with_url(&JsValue::NULL, "", Some(&target.to_string()))
            .unwrap_throw();

        u.set_neq(target);
    });
}

pub fn router(cfg: fn(RouterFactory) -> RouterFactory) -> Dom {
    html!("router", {
        .child_signal(cfg(RouterFactory::default()).build())
    })
}

pub struct RouterFactory {
    path: LocalBoxSignal<'static, Path>,
    routes: Vec<(Route, Rc<Box<dyn Fn(&HandlerContext) -> Option<Dom>>>)>,
}

impl Default for RouterFactory {
    fn default() -> Self {
        Self::from_signal(URL.with(|url| url.signal_cloned()))
    }
}

impl RouterFactory {
    pub fn from_signal(sig: impl Signal<Item = Path> + 'static) -> Self {
        Self {
            path: sig.boxed_local(),
            routes: vec![],
        }
    }

    pub fn with_route<H, Args>(mut self, route: &str, handler: H) -> Self
    where
        H: Handler<Args> + 'static,
        Args: Extract,
    {
        self.routes.push((
            Route::from_str(route).unwrap(),
            Rc::new(Box::new(move |ctx| handler.call(Args::extract(ctx)))),
        ));

        self
    }

    pub fn build(self) -> impl Signal<Item = Option<Dom>> {
        let remainder = Mutable::new(Path::default());

        struct RouterContext {
            route: Route,
            params: Params,
            handler: Rc<Box<dyn Fn(&HandlerContext) -> Option<Dom>>>,
        }

        impl PartialEq for RouterContext {
            fn eq(&self, other: &Self) -> bool {
                self.route == other.route && self.params == other.params
            }
        }

        self.path
            .map({
                let remainder = remainder.clone();
                move |p| {
                    self.routes.iter().find_map(|(route, handler)| {
                        route.matches(&p).map(|m| {
                            remainder.set(m.remainder);
                            RouterContext {
                                route: route.clone(),
                                handler: handler.clone(),
                                params: m.params.clone(),
                            }
                        })
                    })
                }
            })
            .dedupe_map(move |c| {
                c.as_ref().and_then(|c| {
                    (c.handler)(&HandlerContext {
                        params: c.params.clone(),
                        remainder: remainder.clone(),
                    })
                })
            })
    }
}

// thread_local! {
//     static INSTANCE: Router = Router::new(
//         &web_sys::window()
//             .and_then(|w| w.location().pathname().ok())
//             .unwrap_or_default()
//     );
// }

// pub fn router(routes: &[(&str, fn() -> Dom)]) -> impl Signal<Item = Option<Dom>> {
//     let routes: Vec<Route> = routes
//         .iter()
//         .map(|&(path, resolver)| Route::new(path, resolver))
//         .collect();

//     Router::signal_path()
//         .map(move |path| {
//             routes.iter().find_map(|r| {
//                 r.matches(&path).map(|m| {
//                     debug!(?m);
//                     Router::set_remainder(m.remainder());
//                     m
//                 })
//             })
//         })
//         .dedupe_cloned()
//         .map(|m: Option<RouteMatch>| m.map(|m| m.route().resolve()))
// }

// pub fn goto(path: &str) {
//     INSTANCE.with(|r| Router::goto(r, path));
// }

// struct Router {
//     current_path: Mutable<Vec<String>>,
//     remainder: RefCell<Vec<String>>,
// }

// impl Router {
//     fn new(path: &str) -> Self {
//         let segments = split_path(path);

//         Self {
//             current_path: Mutable::new(segments.clone()),
//             remainder: RefCell::new(segments),
//         }
//     }

//     fn goto(&self, path: &str) {
//         let segments = split_path(path);

//         self.remainder.replace(segments.clone());
//         self.current_path.replace(segments);

//         web_sys::window()
//             .unwrap()
//             .history()
//             .unwrap()
//             .push_state_with_url(&JsValue::NULL, "", Some(path))
//             .unwrap();
//     }

//     fn signal_path() -> impl Signal<Item = Vec<String>> {
//         INSTANCE.with(|r| r.current_path.signal_cloned().map(|_| Router::remainder()))
//     }

//     fn remainder() -> Vec<String> {
//         INSTANCE.with(|r| r.remainder.borrow().clone())
//     }

//     fn set_remainder(remainder: Vec<String>) {
//         INSTANCE.with(|r| r.remainder.replace(remainder));
//     }
// }
