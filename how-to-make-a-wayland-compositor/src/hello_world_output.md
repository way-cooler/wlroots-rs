# Analyzing the Wayland protocol
This is the log output from our compositor when it's ran as a nested Wayland instance:

```
{{#include 1-hello-world/sample_output.txt}} 
```

Your output will probably not match exactly, but it should roughly have this output.

## Backend Setup
When `build_auto` is called on `Compositor` it will dynamically detect which backend makes the most sense to spin up. If the compositor is ran in X11 or a Wayland compositor then it will run as a client with all the contents rendered to  a window. If ran on a TTY then it uses the kernel's DRM module. It also possible to [specify a backend directly](http://way-cooler.org/docs/wlroots/compositor/struct.Builder.html#method.build_x11).

```
{{#include 1-hello-world/sample_output.txt:1}}
```

This first line shows which backend was selected. The Wayland backend was selected here, because it was ran in another Wayland instance.<sup>1</sup> 

## Wayland globals
```
{{#include 1-hello-world/sample_output.txt:2:31}}
```

This is a list of all the globals that the parent compositor was advertising. This is output is specific to the Wayland backend. Protocols that are unstable have their names prepended with `z` by convention.

Our compositor also exposes some globals even in this minimal state. Globals are the way clients can start up communication with the Wayland compositor. There are some default ones that come bundled with Wayland, such as `wl_compositor`, and then there are custom ones defined on a per compositor basis. wlroots comes with some popular custom protocols already implemented, but you have to explicitly opt in to using them explicitly in the builder. [xdg shell, for example, is an optional protocol that wasn't used in this example](http://way-cooler.org/docs/wlroots/compositor/struct.Builder.html#method.xdg_shell_manager).

In order to see what globals the toy compositor is advertising you need to use a useful Wayland utility called `weston-info`.<sup>2</sup> It lists the Wayland globals advertised by the current compositor. The current compositor is determined by looking at the `$WAYLAND_DISPLAY` environment variable similar to how in X the current xserver is determined with `$DISPLAY`.

In the log output it prints out the `$WAYLAND_DISPLAY`:

```
{{#include 1-hello-world/sample_output.txt:41}}
```

Lets see what globals are being advertised by our compositor:

```
{{#include 1-hello-world/weston_info_output.txt:1:2}}
```

`weston-info` prints out the name of each exposed interface on a new line along with the highest advertised interface version and the relative, atomically increasing index (confusingly prepended with "name").

`wl_compositor` and `wl_subcompositor` are both standard Wayland interfaces. `wl_subcompositor` is automatically started by wlroots when a `wl_compositor` is used. A `wl_compositor` is the base global that all clients depend on. From this global a client can create a `wl_surface` and a `wl_region`.

A `wl_surface` is a basic building block for drawing and rendering contents to the screen in Wayland. A client needs more than a `wl_surface` in order to render to the screen, but that is the basic object a compositor needs in order to render.<sup>3</sup>

A `wl_region` is the object that allows clients to tell the compositor where, in surface level coordinates<sup>4</sup>, it wants to handle input and where it is rendering content. Where it wants input is very important, but the default is that it accepts input everywhere in the surface. Specifying an area where the client is rendering content is important because it allows the compositor to know that any content behind that doesn't need to be redrawn. As a very simple example of this if there is a moving background on the screen and there is a fullscreen window then there is no need to draw the background saving precious cycles.

The ability to specify only parts of the screen to update is a major feature of Wayland which will be totally ignored until a much later chapter. When starting out it's simple enough to simply redraw the entire screen each time a new frame is available. For non-toy compositors though it is vital that proper damage tracking (as the feature is called) is implemented. It reduces power consumption and makes the compositor faster.

## Seat offerings
```
{{#include 1-hello-world/sample_output.txt:32:33}}
```
Rootson automatically offers the keyboard and mouse to all new windows that appear. This allows input to passthrough directly to the toy compositor, but it also hints at this concept of Wayland "seats".

[A Wayland seat is a collection of inputs devices](https://wayland.freedesktop.org/docs/html/apa.html#protocol-spec-wl_seat) usually handled under the hood by libinput. Seats are created by the compositor, advertised to any new clients including when new input methods are added, and are used to facilitate user input to clients including drag-in-drop.

Seats are necessary to communicate properly with clients and will be explored in a later chapter.

## EGL Setup
```
{{#include 1-hello-world/sample_output.txt:34:40}}
...
{{#include 1-hello-world/sample_output.txt:44:47}}
```

Currently all backends need a renderer in wlroots which is automatically setup when you create one. This output is from the Wayland backend setting up the EGL drawing for rendering. In the future this may change, as the rendering API [is](https://github.com/swaywm/wlroots/issues/774) [in](https://github.com/swaywm/wlroots/issues/558) [flux](https://github.com/swaywm/wlroots/issues/1352).

## Everything after run is called
```
{{#include 1-hello-world/sample_output.txt:41:43}}
```

Everything after these lines, including these lines, is printed to the log after `run` is called. Since there are no clients that connected there is no logging from them and since there are no event handlers nothing else happens.

---
<sup>1</sup> On my machine it was ran in rootson, the wlroots reference compositor, which is why Wayland was selected.

<sup>2</sup> In most Linux distributions this utility is packaged along with weston, the reference Wayland compositor.

<sup>3</sup> Usually a surface is wrapped in a shell. What a shell adds to a `wl_surface` is _context_. Without the proper context a compositor doesn't know if the surface it was just handed by the client is a standalone window, a popup, a background, a status bar, or a cursor to be rendered. All of them need to be handled differently and they are all handled using a dedicated wayland "shell" or a specialized non-shell protocol.

<sup>4</sup> It has to be surface level because clients doesn't know about anything but the content it renders.
