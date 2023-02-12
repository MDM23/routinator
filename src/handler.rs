use std::{cell::RefCell, rc::Rc};

use dominator::Dom;

use crate::{route::Route, Params, RouteContext, Router};

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
    fn extract(router: &RouteContext) -> Self;
}

impl Extract for Params {
    fn extract(ctx: &RouteContext) -> Self {
        ctx.params.clone()
    }
}

impl Extract for Router {
    fn extract(ctx: &RouteContext) -> Self {
        ctx.router.clone()
    }
}

impl Extract for () {
    fn extract(_: &RouteContext) -> Self {}
}

impl<A: Extract> Extract for (A,) {
    fn extract(ctx: &RouteContext) -> Self {
        (A::extract(ctx),)
    }
}

impl<A: Extract, B: Extract> Extract for (A, B) {
    fn extract(ctx: &RouteContext) -> Self {
        (A::extract(ctx), B::extract(ctx))
    }
}
