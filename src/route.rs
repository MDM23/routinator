use std::{ptr, rc::Rc};

use dominator::Dom;

use crate::{
    handler::{Extract, Handler},
    path::Path,
    Params, RouteContext,
};

#[derive(Clone)]
pub struct Route {
    pub path: Path,
    pub(crate) handler: Rc<dyn Fn(&RouteContext) -> Option<Dom>>,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(&*self.handler, &*other.handler)
    }
}

pub trait ToRoute {
    fn to<H, Args>(&self, handler: H) -> Route
    where
        H: Handler<Args> + 'static,
        Args: Extract;

    fn redirect(&self, target: &str) -> Route;
}

impl ToRoute for &str {
    fn to<H, Args>(&self, handler: H) -> Route
    where
        H: Handler<Args> + 'static,
        Args: Extract,
    {
        Route {
            path: Path::new(self),
            handler: Rc::new(move |ctx| handler.call(Args::extract(ctx))),
        }
    }

    fn redirect(&self, target: &str) -> Route {
        let target = target.to_string();
        Route {
            path: Path::new(self),
            handler: Rc::new(move |ctx| {
                ctx.router.goto(&target);
                None
            }),
        }
    }
}

impl Route {
    pub fn matches(&self, path: &Path, skip_segments: usize) -> Option<(usize, Params)> {
        let (mut r, mut p) = (self.path.segments(), path.segments().skip(skip_segments));
        let mut params = Params::default();
        let mut matched_segments: usize = 0;

        loop {
            match (r.next(), p.next()) {
                (Some(r), Some(p)) if r.starts_with(':') => {
                    params.insert(r.strip_prefix(':').unwrap(), p);
                    matched_segments += 1;
                }
                (Some(r), Some(p)) if r == p => {
                    matched_segments += 1;
                }
                (None, _) => {
                    return Some((matched_segments, params));
                }
                _ => {
                    return None;
                }
            }
        }
    }
}
