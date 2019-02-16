# Setting up a `Cursor` and an `output::Layout`
A `Cursor` can be [created at any
time](http://way-cooler.org/docs/wlroots/cursor/struct.Cursor.html#method.create).
Cursors use handles just like all resources provided in wlroots callbacks, so to
use the rest of the methods it must be upgraded.<sup>1</sup>

A `Cursor` can be
[attached](http://way-cooler.org/docs/wlroots/cursor/struct.Cursor.html#method.attach_output_layout)
to an `output::Layout` which will constrain the input to remain within the
region. The layout also keeps track of where the outputs are in relation to each
other, so when the cursor reaches the edge of two outputs it will automatically
warp to the next one.

An `output::Layout` can be [created just like a
`Cursor`](http://way-cooler.org/docs/wlroots/output/layout/struct.Layout.html#method.create).

Here is the new compositor setup code that uses `Cursor`, `output::Layout`, and
`xcursor::Manager`<sup>2</sup>:


```rust
{{#include 3-getting-to-the-point/main.rs:15:52}}
```


# Using `output::Layout`

Outputs can be [added to the layout with
`Layout::add_auto`](http://way-cooler.org/docs/wlroots/output/layout/struct.Layout.html#method.add_auto)
once they are advertised to the compositor:<sup>3</sup> This will allow the
cursor to warp to the next output when the edge is reached between two outputs
in the output layout coordinate space.

```rust
{{#include 3-getting-to-the-point/output.rs:13:}}
```

# Moving the `Pointer`
There is no longer any need to keep track of the current pointer location. This
is tracked by the `Cursor` and can be updated using `move_relative` and `warp`.

We also should update the cursor image when a pointer is added so that the
correct state can be rendered.

Finally here is the code that updates the cursor when the pointer moves:

```rust
{{#include 3-getting-to-the-point/pointer.rs:12:}}
```

---
<sup>1</sup> Unlike most resources in wlroots, `Cursor` lifetimes are entirely
dictated by your code. It will hang around until you [destroy
it](http://way-cooler.org/docs/wlroots/cursor/struct.Handle.html#method.destroy).
However, it acts like the other resources for consistency and because their
lifetimes are tied to other resources. There are two other types like this:
[`output::Layout`](http://way-cooler.org/docs/wlroots/output/layout/struct.Layout.html)
and [`Seat`](http://way-cooler.org/docs/wlroots/seat/struct.Seat.html).

<sup>2</sup> Since the code is getting very long large parts of it will be
elided going forward. The full source can always be found [in the book
repo](https://github.com/swaywm/wlroots-rs/tree/master/how-to-make-a-wayland-compositor/src).

<sup>3</sup> Normally you'd want to [add the output at a specific
point](http://way-cooler.org/docs/wlroots/output/layout/struct.Layout.html#method.add)
in the layout. However this requires user configuration, which is out of the
scope of this book. Currently there is no xrandr equivalent for Wayland.
