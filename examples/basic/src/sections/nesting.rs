use dominator::{html, Dom};
use routinator::Router;

use crate::styles::TAB_BAR;

pub fn nesting(router: Router) -> Dom {
    html!("section", {
        .child(html!("h1", { .text("Nesting") }))

        .child(html!("div", {
            .class(&*TAB_BAR)
            .children(&mut [
                html!("a", {
                    .text("Section A")
                    .apply(router.link("sections/a"))
                }),
                html!("a", {
                    .text("Section B")
                    .apply(router.link("sections/b"))
                }),
            ])
        }))

        .child_signal(
            router
                .route("sections/a", || html!("h2", { .text("Section A") }))
                .route("sections/b", || html!("h2", { .text("Section B") }))
                .default("sections/a")
                .mount()
        )
    })
}
