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

## MS Paint Compositor
TODO this will be fun :)
