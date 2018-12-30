# Exercises

## Russian Doll Compositor
You can trick a Wayland compositor to run as the child of a compositor you're
not currently in by overriding the `WAYLAND_DISPLAY` variable.

Using this, get the toy compositor to run inside the toy compositor.

## Reimplement build_auto
[Explore some of the options for the
`compositor::Builder`](http://way-cooler.org/docs/wlroots/compositor/struct.Builder.html). 

Reimplement `build_auto` using the explicit build functions.

Use `$DISPLAY` to detect when running nested in X11 and `$WAYLAND_DISPLAY` to
detect running nested in Wayland.
