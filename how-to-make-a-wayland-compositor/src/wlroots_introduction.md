# Introduction to wlroots
## What is a compositor framework?
Since Wayland is just a protocol, and a compositor has to do all the things the xserver used to do, a Wayland compositor needs to use more than just Wayland in order to be functional. For example it needs to somehow get access a screens framebuffer in order to render clients. A Wayland compositor is also in charge of forwarding input to clients, like keypresses, and rendering a cursor and moving it when the mouse moves.

The reason it's in charge of all of this is manifold. First it means Wayland can be used in a non desktop setup, such on a phone or in an embedded device where a "cursor" and other such features may not make sense. Second by consolidating the jobs into one process it makes it more efficient because there's more information you can use to make a more informed decision. Finally, Wayland developers are more or less ex-XOrg developers who don't want it to become big, old, and slow like X11 was and by making libwayland itself simple it will more likely survive the years intact.

In order to implement all this additional functionality, most compositors use a few other libraries. The important ones to know are:
* KMS/DRM (Kernal Mode Settings/ Direct Rendering Manager)
  - Interfaces with GPUs of modern video cards to render to screens.
* libinput
  - A library for handling the hardware portions of keyboard, pointers, touch devices, etc. that make up Wayland "seats" (we'll get to what that is later).
* XWayland
  - A compatibility layer that lets you run X11 apps in Wayland. This is optional but is generally used by all compositors.
* Systemd
  - For handling user sessions. This is also optional but broadly supported since logind is standard in most Linux distributions.
* xkbcommon
  - Handling keyboard descriptions and help you process key events.

Not all compositors use frameworks, some of them just use the Wayland and the other libraries directly. Mutter and KWin do not use frameworks. Sway and Way Cooler used to use wlc but they now use wlroots. Fireplace used to use wlc but they now use Smithay, a framework written completely in Rust.

## Which framework will this guide use?
[wlroots](https://github.com/swaywm/wlroots) is the compositor framework that will be used in this book to build a compositor.<sup>1</sup> As of this writing it is the most mature Wayland compositor framework. There are 3 known other compositor frameworks that have problems, which is they will not be discussed except in passing:

* [wlc](https://github.com/Cloudef/wlc)
  - Deprecated. It was found to abstract too much from the Wayland protocol, though it was immensely simpler than wlroots. The time measured to get a working compositor can be measured in hours instead of the expected couple of days or weeks it will take with wlroots. However even basic use cases, such as rendering borders around clients, is difficult to do well in wlc. Some use cases are outright impossible.
* [Smithay](https://smithay.github.io/)
  - A framework written entirely in Rust. Like most things in Rust however it is unstable and attempting to rewrite the entire stack in Rust. This guide (like its readers, hopefully) aims to be practical and that means reusing as much of the ecosystem as we can.
* [libweston](https://gitlab.freedesktop.org/wayland/weston/tree/master/libweston)
  - A library based on the reference Weston compositor, this library is very simplistic. Essentially you're just getting a new flavor of Weston instead of your own compositor, which makes it suffer from the same problems as wlc. As of this writing it's also largely unused outside of Weston.
  
Here is the elevator pitch for wlroots, taken straight from their README:

> Pluggable, composable, unopinionated modules for building a Wayland compositor; or about 50,000 lines of code you were going to write anyway.
> 
> * wlroots provides backends that abstract the underlying display and input hardware, including KMS/DRM, libinput, Wayland, X11, and headless backends, plus any custom backends you choose to write, which can all be created or destroyed at runtime and used in concert with each other.
> * wlroots provides unopinionated, mostly standalone implementations of many Wayland interfaces, both from wayland.xml and various protocol extensions. We also promote the standardization of portable extensions across many compositors.
> * wlroots provides several powerful, standalone, and optional tools that implement components common to many compositors, such as the arrangement of outputs in physical space.
> * wlroots provides an Xwayland abstraction that allows you to have excellent Xwayland support without worrying about writing your own X11 window manager on top of writing your compositor.
> * wlroots provides a renderer abstraction that simple compositors can use to avoid writing GL code directly, but which steps out of the way when your needs demand custom rendering code.
  



---

<sup>1</sup> It is written in C, but there are [safe Rust bindings](https://github.com/swaywm/wlroots-rs) written by me which is what will be used. The only thing the Rust library adds is memory safety and some more structure to the library. All the real features are implemented in wlroots, and if you want you can easily convert this Rust code back to C.
