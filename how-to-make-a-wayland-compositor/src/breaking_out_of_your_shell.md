# Breaking out of your shell
So far the compositor has been talking mostly with system utilities to manage its
resources: DRM to render the frame buffer and initialize the graphics card(s) and
libinput to grab the input devices. This all been important bookkeeping that
must be done in order for a compositor to achieve its main purpose:
communicating with and rendering clients.


Clients are the individual programs spawned by the compositor or its user. It's
the browser, the text editor, the terminal emulator, and any other program the
user might want to use.

There are only three more parts of the pipeline that need to be implemented
before the compositor is usable: spawning clients, communicating with clients,
and rendering clients.

## Spawning clients in the compositor
The `WAYLAND_DISPLAY` environment variable is used by clients to determine
which Wayland compositor to connect to. The environment for the compositor is
automatically updated to have the correct value for this variable once
`Compositor::run` is called. 

To spawn clients with this inherited environment [`Command` from the standard
library can be used](https://doc.rust-lang.org/std/process/struct.Command.html).

In order to keep the compositor simple, the compositor will be configured to
spawn a single client once it starts specified on the command line. By
spawning a useful utility, like a terminal emulator, other programs can be
spawned in the compositor.<sup>1</sup>

## Responding to client requests
Clients use a variety of protocols to communicate with Wayland compositors. Some
of these come standard with Wayland and are automatically handled either by the
Wayland server library itself, or by wlroots (e.g. `wl_output`, `wl_shm`,
`wl_pool`, etc.). 

Others are custom protocols that give the clients more access to resources
controlled by the compositor. Examples of this are found in the [extensions
module](http://way-cooler.org/docs/wlroots/extensions/index.html). They can be
trivially enabled using the `compositor::Builder`.

Like the resources used before, callbacks are registered for requests from
clients and events can be sent to the clients once a handle to them has been
upgraded.

The main object that will be used to communicate with clients is the
[Seat](http://way-cooler.org/docs/wlroots/seat/struct.Seat.html)

## Rendering clients on the outputs
There is a special class of protocol that will be used by clients that want to
render themselves. This class of protocol are called "shell protocols".

The shell protocol that will be focused on in this section is called [XDG
Shell](https://github.com/wayland-project/wayland-protocols/blob/master/stable/xdg-shell/xdg-shell.xml).
It is a protocol that has become standard throughout the Wayland ecosystem.
This protocol wraps `wl_surface`, which in turns wraps a `wl_buffer` which
contains the bytes the compositor should render. 

It adds synchronization of buffers between the compositor and client, as well as
distinguishes between "toplevels" (normal rectangular windows) and "popups" (e.g. from a drop
down).

By using this protocol the compositor will know when it should access the shared
buffer from the client.

---
<sup>1</sup> In a "real world" compositor some sort of configuration file or
script should be used instead of this ad-hoc method. Generally keybindings are
also registered that can spawn programs, depending on the user's preference.
Since this bookkeeping is not strictly related to building a compositor
however, this exercise is left to the reader.
