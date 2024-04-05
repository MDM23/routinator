use dominator::{html, Dom};
use routinator::{Params, Router, ToRoute};

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    dominator::append_dom(&dominator::body(), view());
}

fn view() -> Dom {
    let router = Router::with_browser_url();

    html!("main", {
        .child(html!("ul", {
            .children(vec![
                html!("li", {
                    .child(html!("a", {
                        .text("Element A")
                        .apply(router.link("/details/a"))
                    }))
                }),
                html!("li", {
                    .child(html!("a", {
                        .text("Element B")
                        .apply(router.link("/details/b"))
                    }))
                }),
                html!("li", {
                    .child(html!("a", {
                        .text("Element C")
                        .apply(router.link("/details/c"))
                    }))
                }),
            ])
        }))
        .child(router.mount([
            "/home"        .to(home),
            "/details/:id" .to(details),
            "/"            .redirect("/home"),
        ]))
    })
}

fn home() -> Dom {
    html!("section", {
        .child(html!("h1", {
            .text("Home")
        }))
    })
}

fn details(router: Router, params: Params) -> Dom {
    html!("section", {
        .child(html!("h1", {
            .text(&format!("Details [{}]", params.get("id").unwrap()))
        }))
        .child(html!("a", {
            .text("< Go back")
            .apply(router.link("/list"))
        }))
    })
}
