use dominator::{html, Dom};

const CONTENT: &'static str = r#"
<h1>Quick start</h1>

<p>
    <strong>Routinator</strong> is an easy and flexible routing library that can
    be used together with
    <a href="https://github.com/Pauan/rust-dominator" target="_blank">Dominator</a>.
    Its unique selling points are the dynamic usage of nested routers and that
    it does not redraw / rebuild anything that has not been changed in the route
    tree.
</p>

<h2>Example</h2>

<p>
    To get started, add the latest version of the library to your project:
</p>

<pre>
cargo add routinator
</pre>

<p>
    Add the root router at the desired location. Please note that there can only
    be one root router at a time. To create a different root instance (e.g. to
    handle different states like user and guest), the original one needs to be
    dropped first.
</p>

<pre>
<span class="keyword">use</span> dominator::{Dom, html};
<span class="keyword">use</span> routinator::Router;

<span class="keyword">fn</span> <span class="fn">my_app</span>() -> Dom {
    <span class="keyword">let</span> router = Router::<span class="fn">root</span>();

    html!(<span class="string">"main"</span>, {
        .child_signal(
            router
                .<span class="fn">route</span>(<span class="string">"home"</span>, home_page)
                .<span class="fn">route</span>(<span class="string">"profile"</span>, profile_page)
                .<span class="fn">default</span>(<span class="string">"home"</span>)
                .<span class="fn">mount</span>()
        )
    });
}

<span class="keyword">fn</span> <span class="fn">home_page</span>() -> Dom {
    html!(<span class="string">"section"</span>, {
        .<span class="fn">child</span>(html!(<span class="string">"h1"</span>, {
            .<span class="fn">text</span>(<span class="string">"Welcome!"</span>)
        }))
    })
}

<span class="keyword">fn</span> <span class="fn">profile_page</span>() -> Dom {
    html!(<span class="string">"section"</span>, {
        .<span class="fn">child</span>(html!(<span class="string">"h1"</span>, {
            .<span class="fn">text</span>(<span class="string">"User profile!"</span>)
        }))
    })
}
</pre>

<h2>Route handlers</h2>

<p>
    A route is defined by passing a path and a corresponding handler to the
    <code>route</code> method. You can use any function or closure that returns
    either a <code>Dom</code> or <code>Option&ltDom&gt</code>. It can also
    optionally receive a <code>Router</code> instance as parameter.
</p>

<h2>Router links</h2>
"#;

pub fn quickstart() -> Dom {
    html!("section", { .prop("innerHTML", CONTENT) })
}
