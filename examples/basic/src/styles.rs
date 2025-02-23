use std::sync::LazyLock;

use dominator::{class, pseudo};

pub static CONTENT: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("margin", "0 auto 3rem auto")
        .style("width", "800px")
        .style("max-width", "calc(100vw - 4rem)")
    }
});

pub static NAVIGATION_WRAPPER: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("position", "sticky")
        .style("top", "0")
        .style("padding-top", "2rem")
        .style("margin", "0 -2rem 3rem -2rem")
        .style("background-color", "#FFFFFF")
        .style("box-shadow", "0 2.5rem 1.5rem #FFFFFF")
    }
});

pub static NAVIGATION: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("padding", "0 2rem")
        .style("background-color", "#FFFFFF")
        .style("display", "flex")
        .style("gap", "1rem")
        .style("align-items", "center")
        .style("background-color", "#373737")
        .style("box-shadow", "0px 25px 20px -20px rgba(0, 0, 0, 0.4)")
        .style("border-radius", "1rem")

        .pseudo!(" > a", {
            .style("padding", "0.25rem 1rem")
            .style("border-radius", "0.5rem")
            .style("display", "flex")
            .style("justify-content", "center")
            .style("color", "#DADADA")
        })

        .pseudo!(" > a:hover", {
            .style("color", "#FFFFFF")
        })

        .pseudo!(" > a.routinator-active", {
            .style("background-color", "#F0F0F0")
            .style("color", "#222222")
            .style("cursor", "default")
        })
    }
});

pub static BRAND: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("text-align", "center")
        .style("padding", "1rem 0")
        .style("font-weight", "bold")
        .style("color", "#ff559c")
        .style("cursor", "default")
    }
});

pub static TAB_BAR: LazyLock<String> = LazyLock::new(|| {
    class! {
        .style("background-color", "#F0F0F0")
        .style("border-radius", "0.5rem")
        .style("display", "inline-flex")
        .style("border", "3px #F0F0F0 solid")
        .style("overflow", "hidden")

        .pseudo!(" > a", {
            .style("padding", "0.5rem 1rem")
        })

        .pseudo!(" > a.routinator-active", {
            .style("font-weight", "bold")
            .style("background-color", "#FFFFFF")
        })
    }
});
