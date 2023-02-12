use dominator::{html, Dom};
use routinator::{Router, ToRoute};

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    dominator::append_dom(&dominator::body(), view());
}

fn view() -> Dom {
    let router = Router::with_browser_url();

    html!("main", {
        .child(html!("nav", {
            .children(&mut [
                html!("a", {
                    .text("Home")
                    .apply(router.link_active("home"))
                }),
                html!("a", {
                    .text("Elements")
                    .apply(router.link_active("elements"))
                }),
            ])
        }))
        .child(html!("section", {
            .child_signal(router.mount([
                "/home"     .to(home),
                "/elements" .to(elements),
                "/"         .redirect("home")
            ]))
        }))
    })
}

fn home() -> Dom {
    html!("div", { .text("Home") })
}

fn elements(router: Router) -> Dom {
    let router = router.nest();

    html!("div", {
        .text("Elements")
        .child(html!("input", { .attr("type", "text") }))
        .child(html!("p", {
            .children(&mut [
                html!("a", {
                    .text("Details")
                    .apply(router.link_active("details"))
                }),
                html!("a", {
                    .text("Relations")
                    .apply(router.link_active("relations"))
                }),
            ])
        }))
        .child(html!("section", {
            .child_signal(router.mount([
                "/details"   .to(|| html!("p", { .text("Details") })),
                "/relations" .to(|| html!("p", { .text("Relations") })),
                "/list"      .to(|| html!("p", { .text("List") })),
            ]))
        }))
    })
}
