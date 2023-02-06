use dominator::{html, Dom};
use routinator::{active_router_link, Router};

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    dominator::append_dom(&dominator::body(), view());
}

fn view() -> Dom {
    let mut router = Router::with_browser_url();
    router.add_route("/home", home);
    router.add_route("/elements", elements);
    router.set_default("/home");

    html!("main", {
        .child(html!("nav", {
            .children(&mut [
                html!("a", {
                    .text("Home")
                    .apply(active_router_link(&router, "home"))
                }),
                html!("a", {
                    .text("Elements")
                    .apply(active_router_link(&router, "elements"))
                }),
            ])
        }))
        .child(html!("section", {
            .child_signal(router.mount())
        }))
    })
}

fn home() -> Dom {
    html!("div", { .text("Home") })
}

fn elements(mut router: Router) -> Dom {
    router.add_route("/details", || html!("p", { .text("Details") }));
    router.add_route("/relations", || html!("p", { .text("Relations") }));
    router.add_route("/", || html!("p", { .text("List") }));

    html!("div", {
        .text("Elements")
        .child(html!("p", {
            .children(&mut [
                html!("a", {
                    .text("Details")
                    .apply(active_router_link(&router, "details"))
                }),
                html!("a", {
                    .text("Relations")
                    .apply(active_router_link(&router, "relations"))
                }),
            ])
        }))
        .child(html!("section", {
            .child_signal(router.mount())
        }))
    })
}
