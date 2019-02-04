# Switching TTYs
Implementing the ability to switch TTYs is not much more difficult [once the
relevant function is located on the `Session`
struct](http://way-cooler.org/docs/wlroots/backend/struct.Session.html#method.change_vt).
However getting to that struct from the callback requires explaining wlroot-rs
handles.

> # Handles in wlroots-rs
> Handles represent the wlroots-rs solution to the complicated lifetimes of
> Wayland resources.
>
> In Rust normally you can either own a value or borrow it for some lifetime.
> However, you can't
> "own" a keyboard because you don't control its lifetime. At any point, for
> example, the keyboard
> could be yanked out by the user and then it will need to be cleaned up.
>
> You also can't have these be defined via lifetimes on borrows because
> lifetimes behave like a  compile-time read-write lock on data. That does mean,
> however, that there can be callbacks that takes a borrow, for example:
> ```rust
> /// Callback for when a key is pressed
> fn on_key(keyboard: &Keyboard) {
>     // A Keyboard will be valid here because wlroots is single threaded.
> }
> ```
>
> Unfortunately, now that resource can't escape the callback. Often these
> resources will want to be used beyond this limited scope.
>
> To solve this a `Handle` is used to refer indirectly to resources. Handles are
> essentially thin wrappers around
> [`Weak` smart pointers](https://doc.rust-lang.org/std/rc/struct.Weak.html).
> They can only be accessed in
> callbacks by calling `run` on them, which performs additional safety checks to
> ensure the `Handle` is valid.
>
> In order to make using handles easier there are also two macros that make
> them much easier to use:
> [with_handles](http://way-cooler.org/docs/wlroots/macro.with_handles.html)
> and [wlroots_dehandle](http://way-cooler.org/docs/wlroots_dehandle/macro.wlroots_dehandle.html).
> Either, or neither, can be used. They are only implemented as a convenience.
>
> However, `wlroots_dehandle` will be used later in this book since it is the
> most convenient way to use handles. [So please read its
> documentation](http://way-cooler.org/docs/wlroots_dehandle/macro.wlroots_dehandle.html).
>
> Please read [the handle documentation in order to better understand
> Handles](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html).

## Accessing the Session from the Compositor

A `Session` is [obtained from a
`Backend`](http://way-cooler.org/docs/wlroots/backend/enum.Backend.html#method.get_session).
A `Backend` can be [obtained from a
`&Compositor`](http://way-cooler.org/docs/wlroots/compositor/struct.Compositor.html#method.backend).
To get a reference to the `Compositor` the `compositor_handle` must be upgraded.

When you upgrade a handle it can potentially fail [according to its
signature](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html#method.run).
The [possible error
values](http://way-cooler.org/docs/wlroots/utils/enum.HandleErr.html) indicate
the two requirements for upgrading a handle are:

1. Two handles to the same resource can not be upgraded at the same time. If
   this were allowed there could be two mutable references to the same resource
   which is against Rust's memory model.
2. If the resource behind the handle has been destroyed then the handle can
   never be upgraded again.<sup>1</sup>


Because these errors should not occur for the compositor handle, it is
sufficient to simply `unwrap` the result:

```rust
{{#include 2-goodbye-world/main.rs:31:37}}
{{#include 2-goodbye-world/main.rs:47:}}
```

---
<sup>1</sup> In this case the `Compositor` lives for the life of the compositor,
so it will never be `AlreadyDropped`. This is not the case for other resources,
such as keyboards.
