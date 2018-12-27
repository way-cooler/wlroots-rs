# Analyzing the code
After explaining what each change gives us, I'll then explain what each line of code does.

At the end of each chapter there will be a list of suggestions and challenges which I suggest you at least read over if not try. They exist to encourage you to read through the [wlroots-rs documentation](http://docs.rs/wlroots-rs) and [Wayland documentation](https://wayland.freedesktop.org/docs/html/) in order to better familiarize yourself.

## Logging setup
```rust
{{#include 1-hello-world/main.rs:6}}
```
This line is not strictly necessary for the compositor to run. wlroots (and wlroots-rs) prints a log message each time something interesting happens which is useful for debugging. In general, you should always have this line in your compositor.

The first parameter is the minimum level that is logged. The second parameter is an optional callback that will be called each time a message is logged.

You can log a message using this system by using the `wlr_log!` macro<sup>1</sup>. Here is an example:

```rust
// It has the same syntax as println! or format!
wlr_log!(WLR_DEBUG, "This is an example {:?}", some_struct)
```

The first parameter is the log level you want to log at.


```rust
{{#include 1-hello-world/main.rs:7}}
```
This is the real meat of the program.

This creates a builder for a `Compositor`. There can only be one `Compositor` object per process<sup>2</sup>. The builder is how Wayland globals and their callbacks are set up.

In this case no callbacks are set up the `Compositor` is just immediately built. When you build the compositor, just like you build any object in wlroots-rs, you need to give it user state. In this case, there is no state to store so you can just pass the unit type.

Once the `Compositor` is set up then `run` can be called. This will put it in the main Wayland event loop listening for events and dispatching to the callbacks. It will keep running until `wlroots::terminate` is called. Since we never call it in this compositor, it won't happen until you kill it via a signal.

---
<sup>1</sup> Don't forget to import macros by prepending `#[macro_use]` to `extern crate wlroots`.

<sup>2</sup> wlroots-rs is not designed to be thread safe with its objects. Most objects are `!Send` and `!Sync`.
