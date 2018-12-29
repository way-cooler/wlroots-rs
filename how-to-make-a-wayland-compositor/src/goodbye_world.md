# Goodbye World
The compositor from the previous section has a bug\: it can't be exited from if it is started in DRM. This is a pretty serious bug, one that will be addressed in two ways in this section.

The first is the most extreme, and easiest to implement, option: adding a shut down key sequence. The compositor will be configured so if the user presses `Ctrl+Shift+Escape` it will gracefully terminate with a zero exit status. This will be useful in debugging the compositor as it makes it easy to shut down even in DRM.<sup>1</sup>

The second escape access is a feature that is often taken for granted: the ability to switch TTYs. The standard `Ctrl+Alt+F#` sequence will be implemented to switch TTYs when the compositor is running on DRM. When it's running on another backend it will simply ignore that (since it won't have the proper access controls to do the context switch).

This section will primarily concern itself with [setting up handlers](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html) for the first time, handling [keyboard input](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html), and [learning to use wlroots-rs handles](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html).


# Shutting down the compositor gracefully
Before the compositor can begin listening for keyboard input it needs to listen for keyboards.

For each resource type that can be created there is a manager module that provides a builder and some function signatures for the compositor writer to describe how a resource should be managed. [Here is the input device resource manager module](http://way-cooler.org/docs/wlroots/input/manager/index.html).

To specify that a keyboard should be managed by the compositor a function needs to be defined [matching the keyboard resource manager signature](http://way-cooler.org/docs/wlroots/input/manager/type.KeyboardAdded.html). This function will be later called by wlroots when a keyboard is announced to the compositor through libinput.

Once the function is defined with the necessary signature it needs to be put into the resource builder and the resource builder is passed to the `compositor::Builder`. [Here is the input builder](http://way-cooler.org/docs/wlroots/input/manager/struct.Builder.html) and [here is where the `compositor::Builder` is given an input builder](http://way-cooler.org/docs/wlroots/compositor/struct.Builder.html#method.input_manager).

## A Minimal Keyboard Handler

I will now do the bare minimum to implement the signature needed for a keyboard to be constructed:

```rust
{{#include 2-goodbye-world/main.rs:8:20}}
    None
}
```

With the provided implementation whenever a keyboard is announced wlroots-rs will call `keyboard_added`. Since it unconditionally returns `None` it will not allocate anything for they keyboard resource and it will simple be dropped.

In order to hang on to the resource a handler must be defined and allocated using `Box` to make a trait object. The handler defines how to deal with event the resource can trigger, including [when a key is pressed](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html#method.on_key).

A trait needs a structure to implement on so a [zero-sized](https://doc.rust-lang.org/nomicon/exotic-sizes.html) is used here so there's no overhead: <sup>2</sup>

```rust
{{#include 2-goodbye-world/main.rs:18:23}}
{{#include 2-goodbye-world/main.rs:24:26}}
    // All handler methods have a default implementation that does nothing
    // So because no methods are define here, every event on the keyboard
    // is ignored.
}
```

## Implementing Shutdown
When a key is pressed [this method](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html#method.on_key) receives [the event](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html). The key event has a couple methods but the [most important one is `pressed_keys`](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html#method.pressed_keys). It will provide all the keys as seen by xkb that were pressed when the event fired.

Using the keysyms module from the reexported [xkbcommon crate](https://crates.io/crates/xkbcommon) the list of keys can be iterated over and pattern matched.

The last piece of the puzzle is stopping the compositor. Call [`terminate` to stop](http://way-cooler.org/docs/wlroots/compositor/fn.terminate.html) the compositor. It can be called at any time and will gracefully kill clients, destroy resource managers, and then wind back up the stack to where `run` was called.

Here is the complete code for a compositor that will be able to close itself if there is a keyboard with an escape key:

``` rust
{{#include 2-goodbye-world/main.rs:1:33}}
{{#include 2-goodbye-world/main.rs:42:}}
```

# Switching TTYs
Implementing the ability to switch TTYs is not much more difficult [once the relevant function is located on the `Session` struct](http://way-cooler.org/docs/wlroots/backend/struct.Session.html#method.change_vt). However getting to that struct from the callback requires explaining wlroot-rs handles.

> # Handles in wlroots-rs
> Handles represent the wlroots-rs solution to the complicated lifetimes of wlroots resources.
>
> In Rust normally you can either own a value or borrow it for some lifetime. However, you can't
> "own" a keyboard because you don't control its lifetime. At any point, for example, the keyboard
> could be yanked out by the user and then it will need to be cleaned up.
>
> You also can't have these be defined via lifetimes on borrows because lifetimes behave like a
> compile-time read-write lock on data. That does mean there can have a callback that takes a borrow, for example:
> ```rust
> /// Callback for when a key is pressed
> fn on_key(keyboard: &Keyboard) {
>     // A Keyboard will be valid here because wlroots is single threaded.
> }
> ```
>
> but now that resource can't escape these limited callbacks. That's unfortunate because you want to use the resources
> outside these limited scopes.
>
> To solve this a `Handle` is used to refer indirectly to resources. Handles are essentially thin wrappers around 
> [`Weak` smart pointers](https://doc.rust-lang.org/std/rc/struct.Weak.html). They can only be accessed in 
> callbacks by calling `run` on them, which performs additional safety checks to ensure the `Handle` is valid.
>
> Please read [the handle documentation in order to better understand Handles](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html).

A `Session` is [obtained from a `Backend`](http://way-cooler.org/docs/wlroots/backend/enum.Backend.html#method.get_session). A `Backend` can be [obtained from a `&Compositor`](http://way-cooler.org/docs/wlroots/compositor/struct.Compositor.html#method.backend). To get a reference to the `Compositor` the `compositor_handle` must be upgraded.

When you upgrade a handle there's a change it can fail [according to its signature](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html#method.run). The [possible error values](http://way-cooler.org/docs/wlroots/utils/enum.HandleErr.html) indicate the two requirements for upgrading a handle:

1. Two handles to the same resource can not be upgraded at the same time. If this were allowed there could be two mutable references to the same resource, which is against Rust's memory model.
2. If the resource behind the handle has been destroyed then the handle can never be upgraded again.<sup>3</sup>


Because these errors should not occur for the compositor handle, it is sufficient to simply `unwrap` the result here:

```rust
{{#include 2-goodbye-world/main.rs:26:}}
```

---
<sup>1</sup> Huge caveat to this: if the system is deadlocked, such as by an innocuous `loop {}`, then the compositor can no longer process input including the escape sequence. Either test all of your features in a nested instance (where input can still be processed by the parent system) or have `ssh` as a backup to `pkill` the process.

<sup>2</sup> There's still an allocation, but it will allocate `sizeof(data_struct) + sizeof(vtable) + sizeof(internal_wayland_listeners)`. Even though the size of the data struct is zero there is a non zero cost for the internal wayland listeners and vtable (which allows you to have implementations for the same trait implemented differently on different structs). This is a price you must always pay. [In the future it might be possible to specify static listeners](https://github.com/swaywm/wlroots-rs/issues/238).

<sup>3</sup> In this case the `Compositor` lives for the life of the compositor, so it will never be `AlreadyDropped`. This is not the case for other resources, such as `Keyboard`s.
