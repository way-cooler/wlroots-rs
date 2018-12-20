# Hello World
Each chapter's code contents will have its own folder. The chapter number will prefix it with a name. For example, this chapter's code is stored in `1-hello-world/`. The code can be found [here](https://github.com/swaywm/wlroots-rs/tree/master/how-to-make-a-wayland-compositor/src/).

Only wlroots will be used, no other library. A more useful compositor will want to use other libraries, it is not done here in order to avoid choosing favorites while also being self contained and complete.

>An important note to copy pasters: This document as well as the example code base is under the CC0 license.
>
>So copy and paste liberally, you can use this code as a jumping off point for anything.
>
>**This does _not_ apply to wlroots-rs, which is MIT.**

## Boring setup
wlroots-rs is in crates.io. If you want to install it, simply add this to your Cargo.toml:

```toml
[dependencies]
wlroots = {version = "0.2", features = ["unstable", "static"]}
```

Note that the library is still unstable, and it's changing frequently. So you might want to add it as a git submodule and update it as you see fit.

The static flag statically links the wlroots library to the binary, which is important since only a subset of the library is currently stable. Stable in this case means ABI compatible. There is a stable interface, but it is currently a useless subset of the library.

## A minimal compositor

Here is the smallest, simplest compositor you can make with wlroots:

```rust
{{#include 1-hello-world/main.rs}} ```

This compositor at first appears useless. In fact, it's somewhat dangerous. 

You can compile and run<sup>1</sup> the above in any existing X11 window manager or Wayland compositor and it will run in a nested window.<sup>2</sup> However if you run it in a separate TTY it will use the DRM backend. This is usually the backend that will be used when you're not testing the compositor. If you run it on DRM, you can't escape the compositor.

Because no callbacks were set up the compositor will just keep running forever doing nothing. If you run this in DRM, you need to reboot your computer to escape. You can't even switch TTYs because that's up to the compositor to set up.

This example is a little silly, but it highlights just how much needs to be implemented -- our compositor can't even shut itself off.

---
<sup>1</sup>If you are running on a system with systemd and have the feature enabled (it is by default) it should "just work" for your user. If not, you'll need set the setuid bit on the binary `chmod u+s`

<sup>2</sup>This is a wlroots feature that is built into the `build_auto` function. It is very useful for debugging.

