use dominator::{html, Dom};

const CONTENT: &'static str = r#"
<h1>Parameters</h1>

<p>
    Parameters are dynamic parts of a route and can be defined in the form
    <code>/parameter/:param</code>. When the user navigates to "/parameter/foo"
    for example, the value of the <code>param</code> parameter would be "foo".
    The current value of any parameter within the whole route tree until the
    current point can be obtained from the <code>Router</code> instance:
</p>

<pre>
fn parameter_view(router: Router) -> Dom {
    html!("pre", {
        .text(&format!("The current value of param is: {}", router.param("param")))
    })
}
</pre>
"#;

pub fn parameters() -> Dom {
    html!("section", { .prop("innerHTML", CONTENT) })
}
