mod sections;
mod styles;

use dominator::{html, text};
use routinator::Router;
use styles::{BRAND, CONTENT, NAVIGATION, NAVIGATION_WRAPPER};

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let router = Router::root();

    dominator::append_dom(
        &dominator::body(),
        html!("main", {
            .class(&*CONTENT)
            .child(html!("div", {
                .class(&*NAVIGATION_WRAPPER)
                .child(html!("nav", {
                    .class(&*NAVIGATION)
                    .child(html!("span", {
                        .class(&*BRAND)
                        .child(html!("span", { .text("<") .style("opacity", "0.5") }))
                        .child(text(" Routinator "))
                        .child(html!("span", { .text("/>") .style("opacity", "0.5") }))
                    }))
                    .children(&mut [
                        html!("a", {
                            .text("Quick start")
                            .apply(router.link("quick-start"))
                        }),
                        html!("a", {
                            .text("Parameters")
                            .apply(router.link("parameters"))
                        }),
                        html!("a", {
                            .text("Nesting")
                            .apply(router.link("nesting"))
                        }),
                        html!("a", {
                            .attr("href", "https://github.com/MDM23/routinator")
                            .attr("target", "_blank")
                            .style("margin-left", "auto")
                            .style("padding", "0")
                            .child(html!("img", {
                                .attr("src", "https://img.shields.io/github/stars/MDM23/routinator")
                            }))
                        })
                    ])
                }))
            }))
            .child_signal(
                router
                    .route("quick-start", sections::quickstart)
                    .route("nesting", sections::nesting)
                    .route("parameters", sections::parameters)
                    .default("quick-start")
                    .mount()
            )
        }),
    );
}
