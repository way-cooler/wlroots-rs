// Stable interfaces

#include <wlr/util/edges.h>
#include <wlr/util/log.h>
#include <wlr/util/region.h>

#include <wlr/xcursor.h>

#include <xcursor.h>

#ifdef WLR_USE_UNSTABLE

#include <wlr/backend.h>
#include <wlr/backend/drm.h>
#include <wlr/backend/headless.h>
#include <wlr/backend/interface.h>
#include <wlr/backend/libinput.h>
#include <wlr/backend/multi.h>
#include <wlr/backend/session.h>
#include <wlr/backend/wayland.h>
#include <wlr/backend/headless.h>
#include <wlr/backend/x11.h>
#include <wlr/backend/session/interface.h>

#include <wlr/render/wlr_renderer.h>
#include <wlr/render/egl.h>
#include <wlr/render/gles2.h>
#include <wlr/render/interface.h>
#include <wlr/render/wlr_texture.h>

#include <wlr/types/wlr_box.h>
// NOTE this is stable, but it relies on wlr_box.h which isn't
#include <wlr/types/wlr_matrix.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/types/wlr_cursor.h>
#include <wlr/types/wlr_data_device.h>
#include <wlr/types/wlr_export_dmabuf_v1.h>
#include <wlr/types/wlr_linux_dmabuf_v1.h>
#include <wlr/types/wlr_gtk_primary_selection.h>
#include <wlr/types/wlr_gamma_control_v1.h>
#include <wlr/types/wlr_idle.h>
#include <wlr/types/wlr_idle_inhibit_v1.h>
#include <wlr/types/wlr_input_device.h>
#include <wlr/types/wlr_keyboard.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_output_damage.h>
#include <wlr/types/wlr_pointer.h>
#include <wlr/types/wlr_region.h>
#include <wlr/types/wlr_server_decoration.h>
#include <wlr/types/wlr_screenshooter.h>
#include <wlr/types/wlr_screencopy_v1.h>
#include <wlr/types/wlr_seat.h>
#include <wlr/types/wlr_surface.h>
#include <wlr/types/wlr_tablet_pad.h>
#include <wlr/types/wlr_tablet_tool.h>
#include <wlr/types/wlr_touch.h>
#include <wlr/types/wlr_wl_shell.h>
#include <wlr/types/wlr_xdg_shell_v6.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/types/wlr_xcursor_manager.h>


#include <xwayland.h>
#include <xkbcommon/xkbcommon.h>
#include <pixman.h>

#endif
