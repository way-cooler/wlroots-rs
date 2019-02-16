# How to Make a Wayland Compositor (in Rust)
## What is Wayland?
Wayland is the replacement for the X Window System, colloquially known as X11.

It's recommended that you read [the official Wayland website](https://wayland.freedesktop.org/),
specifically the FAQ, architecture, and documentation, to familiarize yourself with the project.
You should also keep this in your back pocket as you read this book if you need 
clarification on any terminology or concepts you don't understand.

## What is a Wayland Compositor?
If you only learn one thing from this book, it should be that:

**Wayland is not a compositor**

**Wayland is not a window manager**

**Wayland is not a display server**

Rather, **Wayland is a _protocol_ for clients and compositors to speak to each other**

In a typical X11 system, there are two necessary components: the X
server and the window manager. The X server handles rendering and hardware
interactions, and the window manager handles user interaction and window
arrangement. Almost all X11 desktops use the Xorg server as the X server, and
write their own window managers to provide the design and behavior of their
desktop. Gnome, KDE, i3, and AwesomeWM, for example, have a very different
user experience but are all still based on the Xorg server. Fundamentally, the
window manager is an X11 client like any other, and all X11 clients are able to
interact with your desktop in the same way.

A Wayland compositor is different. It is the sole source of authority
on both rendering/hardware *and* window management. The right to arrange
windows on screen and drive user interactions is reserved by the compositor.

Here is a list of compositors that will be referenced later in this book:
* [Way Cooler](http://way-cooler.org)
  - Written by yours truly, Preston Carpenter. It was the first Wayland compositor written in Rust.
* [Fireplace](https://github.com/Drakulix/fireplace)
  - The second Wayland compositor written in Rust, it's goal is to be written completely in Rust including the Wayland implementation.
* [sway](https://swaywm.org/)
  - A clone of i3 by Drew Devault, the first popular tiling Wayland compositor. The wlroots project is overseen by sway.
* [KWin](https://userbase.kde.org/KWin)
  - The KDE Wayland compositor, is the direct continuation of the X11 Plasma desktop.
* [mutter](https://gitlab.gnome.org/GNOME/mutter)
  - The Gnome Wayland compositor, is the direct continuation of the X11 Gnome desktop (which is deprecated).

These compositors will be mentioned either because I know them well enough to
draw experience and stories from in order to help educate you (such as in Way
Cooler's case), because they are doing something unique (in Fireplace's case) or
because they made meaningful or politically significant decisions that have
impacted the ecosystem (sway, mutter, and KWin).

## Prerequisites for understanding reading this book
You should know how to program in Rust. You should have the latest stable 
version of Rust, any edition.

You should also be able to read C. Even though there will be no C in this book 
most of the Wayland ecosystem, including the reference implementation and
wlroots (the framework we will be using), is written in C.
