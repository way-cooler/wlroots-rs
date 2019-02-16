# Exercises

## One Cursor Per Output
If you have multiple outputs lets do something a little different.

Instead of storing the `cursor::Handle` in `CompositorState` try storing it in
the `OutputHandler`. This will give each output its own cursor. You should add a
keybinding to switch what the "current" one is.

## Multiple Input Cursors
If you have multiple input devices hanging around then lets get a little crazy.

Instead of storing the `cursor::Handle` in `CompositorState` try storing it in
the `PointerHandler`. This will allow each pointer to have its own, separate
cursor.

## Configuring Outputs
If you have multiple outputs you probably noticed that the cursor can reach
across all of them. However, it is probably not going across the correct edge
since wlroots has no way to know how the monitors are physically set up in the
world.

Using [the non-auto functions in
`output::Layout`](http://way-cooler.org/docs/wlroots/output/layout/struct.Layout.html#method.add),
and a configuration description of your choice make it possible for the user to
set up their outputs how they like.

> Without endorsing any particular configuration format, it is suggested that
> you nevertheless use [serde](https://github.com/serde-rs/serde) as that is
> the standard way in Rust to encode and decode arbitrary formats.
