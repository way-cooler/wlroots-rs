# Exercises

## Sharing keyboard state

Because the ctrl and shift booleans are implemented on the `KeyboardHandler`
each keyboard gets its own state. That means if you have multiple keyboards
plugged in then the key combination must all be done on the same keyboard.

Modify the compositor to not have this limitation.

Hint: Instead of a global, try replacing the state passed to the
`compositor::Builder`. The compositor can be
[downcasted](http://way-cooler.org/docs/wlroots/compositor/struct.Compositor.html#method.downcast)
to this state.
