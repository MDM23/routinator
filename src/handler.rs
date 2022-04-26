use dominator::Dom;
use futures_signals::signal::Mutable;

use crate::path::{Params, Path};
use crate::RouterFactory;

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

pub struct HandlerContext {
    pub params: Params,
    pub remainder: Mutable<Path>,
}

pub trait Extract {
    fn extract(ctx: &HandlerContext) -> Self;
}

impl Extract for Params {
    fn extract(ctx: &HandlerContext) -> Self {
        ctx.params.clone()
    }
}

impl Extract for RouterFactory {
    fn extract(ctx: &HandlerContext) -> Self {
        RouterFactory::from_signal(ctx.remainder.signal_cloned())
    }
}

impl Extract for () {
    fn extract(_: &HandlerContext) -> Self {}
}

impl<A: Extract> Extract for (A,) {
    fn extract(ctx: &HandlerContext) -> Self {
        (A::extract(ctx),)
    }
}

impl<A: Extract, B: Extract> Extract for (A, B) {
    fn extract(ctx: &HandlerContext) -> Self {
        (A::extract(ctx), B::extract(ctx))
    }
}
