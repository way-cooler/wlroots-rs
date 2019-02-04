# TODO
I don't like this first section too much, might want to shorten it a bit and
shove it in later subsections of this chapter.

I don't think I should start with `wlr_cursor`, lets try to add `wlr_pointer` and
see that it doesn't work so we need to use `wlr_output` and `wlr_output_cursor`. 

But then in the next section show that it doesn't handle multiple outputs and
other things very well.

# Getting to the Point
Now that the compositor can be started stopped successfully, it's about time
that it actually starts doing what it's designed to do: composite some stuff.

There are, generally speaking, two types of resources the compositor will
render: contents of buffers owned by the compositor and contents of buffers
owned by clients. 

Client provided buffers are more complicated to handle in general since there
need to be a certain amount of synchronization that needs to happen before the
compositor can start rendering what is in the buffer. This will be dealt with in
a later chapter since clients deserve their own chapter.

Between different compositors the amount of compositor-owned buffers varies.
For example, some compositors render their own background, status bars, and
window decorations whilst others leave all that up to clients. Neither is more
right or wrong than the other, but each leads to certain design trade-offs.
However, at least among desktop compositors <sup>1</sup>, there is at least one compositor
owned object they all generally have: the cursor.

> Side note: there is one caveat to the previous claim about cursors. 
>
> When a mouse pointer is over a client the buffer for the cursor is provided by
> the client. When a mouse pointer is not over a client, or if the client
> doesn't provide a cursor buffer, then it is up to the compositor to provide
> its own cursor to render.
>
> Ultimately the compositor has to make a decision on whether or not to render
> cursors, but where the buffer comes from can differ depending on the pointer's
> location and whether or not the compositor will honor the client's request.


# Cursors vs Pointers in Wayland


---
<sup>1</sup> Remember that Wayland was designed to be flexible. A compositor
could be running in an embedded, touch screen environment, on a phone, or even
in a VR headset. In none of those environments does a "cursor" make sense.
