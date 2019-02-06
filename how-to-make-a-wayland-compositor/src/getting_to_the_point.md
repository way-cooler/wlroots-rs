# Getting to the Point
Now that the compositor can be started and stopped successfully, it's about time
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

However, at least among desktop compositors, there is at least one 
compositor owned object they all generally have to render: the cursor.

As simple as that sounds, it's actually very complicated rendering a cursor
correctly. Thankfully wlroots makes it much easier.

[Suggested reading for an in-depth dive on how input handling works in
wlroots](https://drewdevault.com/2018/07/17/Input-handling-in-wlroots.html)

