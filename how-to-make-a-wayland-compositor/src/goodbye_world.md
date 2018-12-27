# Goodbye World
The compositor from the previous section has a bug\: it can't be exited from if it is started in DRM. This is a pretty serious bug, one that will be addressed in two ways in this section.

The first is the most extreme, and easiest to implement, option: adding a shut down key sequence. The compositor will be configured so if the user presses `Ctrl+Shift+Escape` it will gracefully terminate with a zero exit status. This will be useful in debugging the compositor as it makes it easy to shut down even in DRM.<sup>1</sup>

The second escape access is a feature that is often taken for granted: the ability to switch TTYs. The standard `Ctrl+Alt+F#` sequence will be implemented to switch TTYs when the compositor is running on DRM. When it's running on another backend it will simply ignore that (since it won't have the proper access controls to do the context switch).

This section will primarily concern itself with [setting up handlers](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html) for the first time, handling [keyboard input](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html), and [learning to use wlroots-rs handles](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html).


# Shutting down the compositor gracefully
Before the compositor can begin listening for keyboard input it needs to listen for keyboards. wlroots-rs provides various "manager" handler traits that describe how to deal with new resources, such as input, when it is created by an external source. For example there is one for [new input devices](http://way-cooler.org/docs/wlroots/input/trait.ManagerHandler.html).

When a specific device is added, [such as a keyboard](http://way-cooler.org/docs/wlroots/input/trait.ManagerHandler.html#method.keyboard_added), you can specify for wlroots-rs to allocate a new object to keep track of that device and respond to its events by returning a handler for that type of device. The default implementation for all of these allocating functions is to not allocate (e.g. return `None`) so there's no need to implement the ones you don't care about.

Here's the code that will automatically listen for new keyboards and allocate a new object for each keyboard that is announced:

```rust
{{#include 2-goodbye-world/main.rs:17:26}}
```

Since Rust traits need a struct to implement on, a [zero-sized](https://doc.rust-lang.org/nomicon/exotic-sizes.html) is used here so there's no overhead. <sup>2</sup>

---
<sup>1</sup> Huge caveat to this: if the system is deadlocked, such as by an innocuous `loop {}`, then the compositor can no longer process input including the escape sequence. Either test all of your features in a nested instance (where input can still be processed by the parent system) or have `ssh` as a backup to `pkill` the process.

<sup>2</sup> There's still an allocation, but it will allocate `sizeof(data_struct) + sizeof(vtable) + sizeof(internal_wayland_listeners)`. Even though the size of the data struct is zero there is a non zero cost for the internal wayland listeners and vtable (which allows you to have implementations for the same trait implemented differently on different structs). This is a price you must always pay. [In the future it might be possible to specify static listeners](https://github.com/swaywm/wlroots-rs/issues/238).
