# Goodbye World
The compositor from the previous section has a bug\: it can't be exited from if
it is started in DRM. This is a pretty serious bug, one that will be addressed
in two ways in this section.

### Gracefully shutting Down

The first is the most extreme, and easiest to implement, option: adding a shut
down key sequence. The compositor will be configured so if the user presses
`Ctrl+Shift+Escape` it will gracefully terminate with a zero exit status. This
will be useful in debugging the compositor as it makes it easy to shut down even
in DRM.<sup>1</sup>

### TTY switching

The second escape access is a feature that is often taken for granted: the
ability to switch TTYs. The standard `Ctrl+Alt+F#` sequence will be implemented
to switch TTYs when the compositor is running on DRM. When it's running on
another backend it will simply ignore that (since it won't have the proper
access controls to do the context switch).

## What you'll learn

This chapter will primarily concern itself with [setting up
handlers](http://way-cooler.org/docs/wlroots/input/keyboard/trait.Handler.html)
for the first time, handling [keyboard
input](http://way-cooler.org/docs/wlroots/input/keyboard/event/struct.Key.html),
and [learning to use wlroots-rs handles](http://way-cooler.org/docs/wlroots/utils/struct.Handle.html).
