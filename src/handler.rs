use dominator::Dom;

use crate::{
    path::{Params, PathMatch},
    ReadOnlyRouter, Router,
};

pub trait Handler<Args> {
    fn call(&self, args: Args) -> Option<Dom>;
}

pub trait HandlerResult {
    fn get(self) -> Option<Dom>;
}

impl HandlerResult for Dom {
    fn get(self) -> Option<Dom> {
        Some(self)
    }
}

impl HandlerResult for Option<Dom> {
    fn get(self) -> Option<Dom> {
        self
    }
}

impl<F, R> Handler<()> for F
where
    F: Fn() -> R,
    R: HandlerResult,
{
    fn call(&self, _args: ()) -> Option<Dom> {
        (self)().get()
    }
}

impl<F, R, A> Handler<(A,)> for F
where
    F: Fn(A) -> R,
    R: HandlerResult,
{
    fn call(&self, args: (A,)) -> Option<Dom> {
        (self)(args.0).get()
    }
}

impl<F, R, A, B> Handler<(A, B)> for F
where
    F: Fn(A, B) -> R,
    R: HandlerResult,
{
    fn call(&self, args: (A, B)) -> Option<Dom> {
        (self)(args.0, args.1).get()
    }
}

pub trait Extract {
    fn extract(router: &ReadOnlyRouter, mtch: &PathMatch) -> Self;
}

impl Extract for Params {
    fn extract(_: &ReadOnlyRouter, mtch: &PathMatch) -> Self {
        mtch.params.clone()
    }
}

impl Extract for Router {
    fn extract(router: &ReadOnlyRouter, mtch: &PathMatch) -> Self {
        Router {
            path: router.path.clone(),
            path_offset: mtch.segments.len(),
            routes: Vec::new(),
            default: None,
            on_change: None,
        }
    }
}

impl Extract for ReadOnlyRouter {
    fn extract(router: &ReadOnlyRouter, _: &PathMatch) -> Self {
        router.clone()
    }
}

impl Extract for () {
    fn extract(_: &ReadOnlyRouter, _: &PathMatch) -> Self {}
}

impl<A: Extract> Extract for (A,) {
    fn extract(router: &ReadOnlyRouter, mtch: &PathMatch) -> Self {
        (A::extract(router, mtch),)
    }
}

impl<A: Extract, B: Extract> Extract for (A, B) {
    fn extract(router: &ReadOnlyRouter, mtch: &PathMatch) -> Self {
        (A::extract(router, mtch), B::extract(router, mtch))
    }
}
