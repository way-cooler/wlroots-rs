# How to Make a Wayland Compositor (in Rust)
## What is Wayland?
Wayland is the replacement for the X Window System, colloquially known as X11. 

Similar to how the most popular implementation of X11 is XOrg the most popular version of Wayland is the reference implementation. Other versions exist, including one [written in Rust](https://github.com/Smithay/wayland-rs).

It's highly suggested you read everything [on the official Wayland site](https://wayland.freedesktop.org/), specifically the FAQ, architecture page, and the documentation. You can also just read along here and consult that site when you come across a term or process you don't understand.

## What is a Wayland Compositor?
If there's only one thing you learn from this book, it should be this:

**Wayland is not a compositor**

**Wayland is not a window manager**

**Wayland is not a display server**

**Wayland is a _protocol_ for clients and compositors to speak to each other**

[This is required reading before you proceed.](https://wayland.freedesktop.org/architecture.html) (feel free to ignore the hardware and rendering sections, focus on the diagrams).

In X11 there are two main components that are normally used to make a functioning system: the xserver and a window manager (which is just a client to the xserver). Technically all you need is the xserver, but the window manager helps make it usable. Most variations of X11 systems are simply swapping out what the window manager is. For example, Gnome, KDE, i3, and AwesomeWM are all examples of very different ways to build a functioning X11 system. Across all of these, they generally will use the same xserver.

A Wayland compositor is like a window manager and the xserver bundled into one. Like in X11 it is in charge of putting other windows in their proper place, but unlike X11 _only_ the compositor has the power to do this. In X11 a window manager is no more privileged than any other client. In Wayland the compositor is the master and the clients are the slaves.

Here is a list of compositors that will be referenced later in this book:
* [Way Cooler](http://way-cooler.org)
  - Written by me, Preston Carpenter. It was the first Wayland compositor written in Rust.
* [Fireplace](https://github.com/Drakulix/fireplace)
  - The second Wayland compositor written in Rust, it's goal is to be written completely in Rust including the Wayland implementation.
* [Sway](https://swaywm.org/)
  - A clone of i3 by sircmpwn, the first popular tiling Wayland compositor. Because of Sway wlroots was made.
* [KWin](https://userbase.kde.org/KWin)
  - The KDE Wayland compositor, is the direct continuation of the X11 Plasma desktop (which is deprecated).
* [Mutter](https://gitlab.gnome.org/GNOME/mutter)
  - The Gnome Wayland compositor, is the direct continuation of the X11 Gnome desktop (which is deprecated).

These compositors will be mentioned either because I know them well enough to draw experience and stories from in order to help educate you (such as in Way Cooler's case), because they are doing something unique (in Fireplace's case) or because they made meaningful or politically significant decisions that have impacted the ecosystem (Sway, Mutter, and KWin).

## Prerequisites to reading this book
You should know how to program in Rust. You should have the latest stable version of Rust, any edition.

You should also be able to read C. Even though there will be no C in this book most of the Wayland ecosystem, including the reference implementation and wlroots (the framework we will be using), is written in C.
