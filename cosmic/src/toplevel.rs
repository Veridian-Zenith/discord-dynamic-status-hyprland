use crate::AppState;
use common::logger::Logger;
use cosmic_protocol::zcosmic_toplevel_info::zcosmic_toplevel_handle_v1::{
    self, ZcosmicToplevelHandleV1,
};
use cosmic_protocol::zcosmic_toplevel_info::zcosmic_toplevel_info_v1::{
    self, ZcosmicToplevelInfoV1,
};
use wayland_client::{Dispatch, Proxy, QueueHandle, event_created_child};

const STATE_ACTIVATED: u32 = 2;

impl Dispatch<ZcosmicToplevelInfoV1, ()> for AppState {
    event_created_child!(AppState, ZcosmicToplevelInfoV1, [
        0 => (ZcosmicToplevelHandleV1, ()),
    ]);

    fn event(
        state: &mut Self,
        _proxy: &ZcosmicToplevelInfoV1,
        event: <ZcosmicToplevelInfoV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zcosmic_toplevel_info_v1::Event::Toplevel { toplevel } => {
                let id = toplevel.id().protocol_id();
                state.toplevel_state.upsert_toplevel(id);
                Logger::log(&format!("New toplevel (id={})", id));
            }
            zcosmic_toplevel_info_v1::Event::Finished => {
                Logger::log("Toplevel info finished");
            }
            _ => {}
        }
    }
}

impl Dispatch<ZcosmicToplevelHandleV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &ZcosmicToplevelHandleV1,
        event: <ZcosmicToplevelHandleV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let obj_id = proxy.id().protocol_id();

        match event {
            zcosmic_toplevel_handle_v1::Event::State { state: states } => {
                let activated = states
                    .chunks_exact(4)
                    .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .any(|s| s == STATE_ACTIVATED);

                Logger::log(&format!("State (id={}): activated={}", obj_id, activated));

                if let Some(info) = state.toplevel_state.toplevels.get_mut(&obj_id) {
                    info.activated = activated;
                }
            }
            zcosmic_toplevel_handle_v1::Event::Title { title } => {
                Logger::log(&format!("Title (id={}): {}", obj_id, title));
                if let Some(info) = state.toplevel_state.toplevels.get_mut(&obj_id) {
                    info.title = Some(title);
                }
            }
            zcosmic_toplevel_handle_v1::Event::AppId { app_id } => {
                Logger::log(&format!("AppId (id={}): {}", obj_id, app_id));
                if let Some(info) = state.toplevel_state.toplevels.get_mut(&obj_id) {
                    info.app_id = Some(app_id);
                }
            }
            zcosmic_toplevel_handle_v1::Event::Done => {
                Logger::log(&format!("Done (id={})", obj_id));
                state.check_focus_and_notify();
            }
            zcosmic_toplevel_handle_v1::Event::Closed => {
                Logger::log(&format!("Closed (id={})", obj_id));
                state.toplevel_state.toplevels.remove(&obj_id);
                state.check_focus_and_notify();
            }
            _ => {}
        }
    }
}
