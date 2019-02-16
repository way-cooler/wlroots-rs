# A basic cursor
There are two main problems that need to be solved to handle a single cursor on
the screen:<sup>1</sup>

1. Keeping track of where the mouse is.
2. Rendering the mouse where it is.

The first problem can be solved with two numbers and an event listener. The
second can be solved with [xcursor](ftp://www.x.org/pub/X11R7.7/doc/man/man3/Xcursor.3.xhtml).

## Keeping track of the mouse
In order to keep track of the mouse a new input device will need to be managed
by the compositor. For this purpose wlroots provides a
[Pointer](http://way-cooler.org/docs/wlroots/input/pointer/index.html). It
abstracts over all types of common mouse input<sup>2</sup>, courtesy of libinput.

Like Keyboard, a Pointer is instantiated using the input builder:

```rust
struct PointerHandler;

impl pointer::Handler for PointerHandler {
    // By default, all events are ignored
}

fn pointer_added(_compositor_handle: compositor::Handle,
                 _pointer_handle: pointer::Handle)
                 -> Option<Box<pointer::Handler>> {
    Some(Box::new(PointerHandler))
}


fn main() {
    let input_builder = wlroots::input::manager::Builder::default()
        .pointer_added(pointer_added)
    // Other setup elided...
}
```

The event that is emitted when the mouse is moved is [the motion
event](http://way-cooler.org/docs/wlroots/input/pointer/event/struct.Motion.html).
This event is provided in the [`pointer::Handler::on_motion`
callback](http://way-cooler.org/docs/wlroots/input/pointer/trait.Handler.html#method.on_motion).
This event provides deltas corresponding to the movement amount. By keeping a
running sum, the absolute position of the mouse, in output coordinates, can be
determined.

> # Different coordinate types
> Throughout this book different coordinate types will be used. Each coordinate
> is a number representing a position inside some viewport.
>
> The main types of coordinates used are:
> 1. Output coordinates
> 2. Output layout coordinates
> 3. View coordinates
>
> They are generally distinguished in the docs and code by prepending a letter
> to the variable name. For example `lx` is the x position in relation to the
> output layout.
>
> The origin point will always be in the top left corner.

The coordinates can be stored in the `PointerHandler` and updated on each event:

```rust
struct PointerHandler {
    /// The x coordinate in relation to the output.
    ox: f64,
    /// The y coordinate in relation to the output.
    oy: f64
}

impl pointer::Handler for PointerHandler {
    fn on_motion(&mut self,
                 compositor_handle: compositor::Handle,
                 _pointer_handle: pointer::Handle,
                 motion_event: &pointer::event::Motion) {
        let (delta_x, delta_y) = motion_event.delta();
        self.x += delta_x;
        self.y += delta_y;
    }
}
```


## Rendering a mouse with xcursor
Now that the mouse position can be tracked it's time to render it to the screen.

The xcursor library doesn't render anything itself, it just provides images from
the system. In a typical desktop environment the cursor changes its icon
depending on what's under it, which requires a manager to keep track of all
these types of images.

In fact, in Wayland clients can dictate what the cursor looks like. When a
client is receiving input from the mouse it can provide its own cursor image.
Though the compositor is not obligated to use this mouse, it is common to do so.

An [xcursor
theme](http://way-cooler.org/docs/wlroots/cursor/xcursor/struct.Theme.html) can
be
[loaded](http://way-cooler.org/docs/wlroots/cursor/xcursor/struct.Theme.html#method.load_theme)
at the start of the program and stored in the `CompositorState`.<sup>3</sup>

Now that the image has been obtained there needs to be something to render it
onto. Thus far the compositor has not been aware of any outputs, beyond the auto
detection it does during backend setup. 

### Outputs
An [Output](http://way-cooler.org/docs/wlroots/output/struct.Output.html)
represents a rectangular view port on which clients and other content are
rendered. Generally this means a monitor plugged into the computer, though if
the Wayland or X11 backends are used then it will instead be a window as a
client to the host system.


Setting up an output is done in the same as setting up an input. There is only
one crucial difference: when setting up an output there needs to be a [mode set
for the output using the builder passed into the
function](http://way-cooler.org/docs/wlroots/output/struct.Builder.html).


```rust
struct OutputHandler;

impl output::Handler for OutputHandler {}

    fn output_added<'output>(compositor: compositor::Handle,
                             builder: output::Builder<'output>)
                             -> Option<output::BuilderResult<'output>> {
    Some(builder.build_best_mode(OutputHandler))
}

fn main() {
    let output_builder = wlroots::output::manager::Builder::default()
        .output_added(output_added);
    let compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
}
```

Rendering is done in the [on frame
callback](http://way-cooler.org/docs/wlroots/output/trait.Handler.html#method.on_frame),
however for cursors this is not necessary. wlroots provides a special [output
cursor](http://way-cooler.org/docs/wlroots/output/struct.Cursor.html) which
abstracts over rendering a cursor. This is because many backends support
"hardware" cursors. This is a feature provided by GPUs that allow moving a
cursor around the screen without redrawing anything underneath it.

If hardware cursors aren't supported the `output::Cursor` will revert to using
software cursors automatically.

Using this new type this is a complete basic cursor implementation with wlroots:

```rust
struct OutputHandler;

impl output::Handler for OutputHandler {}

#[wlroots_dehandle]
fn output_added<'output>(compositor: compositor::Handle,
                             builder: output::Builder<'output>)
                             -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(OutputHandler);
    {
        #[dehandle] let compositor = compositor;
        #[dehandle] let output = &result.output;
        let state: &mut CompositorState = compositor.downcast();
        let mut cursor = output::Cursor::new(output)
            .expect("Could not create output cursor");
        let xcursor = state.theme.get_cursor("left_ptr".into())
            .expect("Could not load default cursor set");
        let image: wlroots::render::Image = xcursor.image(0)
            .expect("xcursor had no images").into();
        cursor.set_image(&image)
            .expect("Could not set cursor image");
        state.cursor = Some(cursor);
    }
    Some(result)
}

struct PointerHandler;

impl pointer::Handler for PointerHandler {
    #[wlroots_dehandle]
    fn on_motion(&mut self,
                 compositor_handle: compositor::Handle,
                 _pointer_handle: pointer::Handle,
                 motion_event: &pointer::event::Motion) {
        #[dehandle] let compositor = compositor_handle;
        let &mut CompositorState { ref mut cursor, .. } = compositor.downcast();
        if let Some(cursor) = cursor.as_mut() {
            let (delta_x, delta_y) = motion_event.delta();
            let (cur_x, cur_y) = cursor.coords();
            cursor.move_to(cur_x + delta_x, cur_y + delta_y)
                .expect("Could not move cursor");
        }
    }
}


fn pointer_added(_compositor_handle: compositor::Handle,
                     _pointer_handle: pointer::Handle)
                     -> Option<Box<pointer::Handler>> {
    Some(Box::new(PointerHandler))
}

struct CompositorState {
    theme: xcursor::Theme,
    cursor: Option<wlroots::output::Cursor>
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let theme = xcursor::Theme::load_theme(None, 16)
        .expect("Could not create xcursor manager");
    let output_builder = wlroots::output::manager::Builder::default()
        .output_added(output_added);
    let input_builder = wlroots::input::manager::Builder::default()
        .pointer_added(pointer_added)
        .keyboard_added(keyboard_added);
    let compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(CompositorState { theme, cursor: None });
    compositor.run();
}
```

> # Box of the Socratic Teaching Style
> Before continuing, I suggest you think for a moment on some complications or
> desirable features we ignored in this design. Try using the above example
> yourself and see if there's any bugs in it.
>
> When considering features for your compositor, it's important to consider
> setups different from your own, which can help ensure your compositor is
> flexible enough to withstand the "real world".

## Problems with this approach
Unfortunately, this is a very bad solution to the problem. One problem that's
obvious if the above example is tried is that there are no bounds checks for
when the cursor goes outside the output.

Another problem, which is more difficult to solve, is when multiple outputs are
connected. When this happens only the last one gets a cursor and the others are
inaccessible. This is because each output is its own buffer and have no relation
to the others. So a relationship between outputs must be establish where it's
possible to "move" to another output when the edge of another is reached.

Finally, this solution also doesn't address how to react to drawing tablets or
touch screens.

These problems are all very complicated, not to mention very boring. In order to
solve them wlroots provides two semi-connected abstractions: a
[Cursor](http://way-cooler.org/docs/wlroots/cursor/struct.Cursor.html) and a
[Layout](http://way-cooler.org/docs/wlroots/output/layout/struct.Layout.html).


---

<sup>1</sup> Note that in Wayland there are no restrictions on the number of
cursors. Multiple cursors can be rendered at the same time and can be controlled
by any number of other input devices (including fake ones).

<sup>2</sup> Non-exhaustive list: common two/three button mice, multi-button
mice, trackpoints, touchpads, and trackballs.

<sup>3</sup> Generally a [theme
manager](http://way-cooler.org/docs/wlroots/cursor/xcursor/struct.Manager.html)
is used to generate these themes, but this will be ignored for now.
