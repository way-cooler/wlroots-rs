# A graceful shutdown
Before the compositor can begin listening for keyboard input it needs to listen 
for keyboards.

For each resource type that can be created there is a manager module that
provides a builder and some function signatures for the compositor writer to
describe how a resource should be managed. 
[Here is the input device resource manager module](http://way-cooler.org/docs/wlroots/input/manager/index.html).

To specify that a keyboard should be managed by the compositor a function needs
to be defined [matching the keyboard resource manager
signature](http://way-cooler.org/docs/wlroots/input/manager/type.KeyboardAdded.html).
This function will be later called by wlroots when a keyboard is announced to
the compositor through libinput.

Once the function is defined with the necessary signature it needs to be 
[put into the resource builder](http://way-cooler.org/docs/wlroots/input/manager/struct.Builder.html)
and the resource builder [passed to the
`compositor::Builder`](http://way-cooler.org/docs/wlroots/compositor/struct.Builder.html#method.input_manager).

## A Minimal Keyboard Handler

This is the simplest function that implements the signature:

```rust
{{#include 2-goodbye-world/main.rs:19:21}}
    None
}
```

This is how it's passed to the builder:

```rust
{{#include 2-goodbye-world/main.rs:9:17}}
```

With the provided implementation whenever a keyboard is announced wlroots-rs
will call `keyboard_added`. Since the function unconditionally returns `None` a
keyboard resource handler will never be allocated and the resource will be dropped.

## Holding on to the resource

In order to hang on to the resource a handler must be defined and allocated
using `Box` to make a trait object. The handler defines how to deal with event
the resource can trigger, including [when a key is
pressed](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html#method.on_key).

Since a resource handler is a trait object each resource handler has a piece of
state it holds between callbacks separate from the other resources. It is here
where the "shift" and "ctrl" pressed state will be held:

```rust
{{#include 2-goodbye-world/main.rs:25:29}}
```

In order to be able to return a `Box`-ed version of this struct in
the `keyboard_added` function `keyboard::Handler` will need to be implemented:

```rust
{{#include 2-goodbye-world/main.rs:19:23}}

{{#include 2-goodbye-world/main.rs:31}}
    // All handler methods have a default implementation that does nothing.
    // So because no methods are define here, every event on the keyboard
    // is ignored.
}
```

## Listening for keyboard modifiers
When a key is pressed [this
method](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html#method.on_key)
receives [the
event](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html).
The key event has a couple methods but the [most important one is
`pressed_keys`](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html#method.pressed_keys).
It will provide all the keys as seen by xkb that were pressed when the event
fired.

You can also see if the key was pressed or not [with 
`key_state`](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html#method.key_state).
This is necessary to determine the boolean state in `KeyboardHandler`.

Using the keysyms module from the reexported [xkbcommon
crate](https://crates.io/crates/xkbcommon) the list of keys can be iterated over
and pattern matched. Here is all that put together to toggle the booleans
when the appropriate keys are pressed:

```rust
{{#include 2-goodbye-world/main.rs:31:41}}
{{#include 2-goodbye-world/main.rs:55:}}
```

The last piece of the puzzle to stopping the compositor is the [`terminate`
function](http://way-cooler.org/docs/wlroots/compositor/fn.terminate.html). It
can be called at any time and will gracefully kill clients,
destroy resource managers, and then wind back up the stack to where `run` was
called.

Here is the complete code for a compositor that will close itself when
`Ctrl+Shift+Escape` is pressed:

``` rust
{{#include 2-goodbye-world/main.rs:1:46}}
{{#include 2-goodbye-world/main.rs:55:}}
```

---

<sup>1</sup> Huge caveat to this: if the system is deadlocked, such as by an
innocuous `loop {}`, then the compositor can no longer process input including
the escape sequence. Either test all of your features in a nested instance
(where input can still be processed by the parent system) or have `ssh` as a
backup to `pkill` the process.
