use crate::AppState;
use common::logger::Logger;
use cosmic_protocol::zcosmic_toplevel_info::zcosmic_toplevel_handle_v1::{
    self, ZcosmicToplevelHandleV1,
};
use cosmic_protocol::zcosmic_toplevel_info::zcosmic_toplevel_info_v1::{
    self, ZcosmicToplevelInfoV1,
};
use wayland_client::{Dispatch, Proxy, QueueHandle};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1::{self, ExtForeignToplevelHandleV1},
    ext_foreign_toplevel_list_v1::{self, ExtForeignToplevelListV1},
};

const STATE_ACTIVATED: u32 = 2;

#[derive(Clone, Copy, Debug)]
pub struct CosmicHandleData {
    pub ext_id: u32,
}

impl Dispatch<ExtForeignToplevelListV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ExtForeignToplevelListV1,
        event: <ExtForeignToplevelListV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            ext_foreign_toplevel_list_v1::Event::Toplevel { toplevel } => {
                let id = toplevel.id().protocol_id();
                state.toplevel_state.upsert_toplevel(id);
                Logger::log(&format!("New ext toplevel (id={})", id));

                if let Some(cosmic_mgr) = &state.cosmic_toplevel_manager {
                    Logger::log(&format!(
                        "Requesting cosmic toplevel for ext handle (id={})",
                        id
                    ));
                    let cosmic_handle: ZcosmicToplevelHandleV1 = cosmic_mgr.get_cosmic_toplevel(
                        &toplevel,
                        qhandle,
                        CosmicHandleData { ext_id: id },
                    );
                    let cosmic_id = cosmic_handle.id().protocol_id();
                    state.toplevel_state.ext_to_cosmic.insert(id, cosmic_id);
                    Logger::log(&format!("Mapped ext {} -> cosmic {}", id, cosmic_id));
                }
            }
            ext_foreign_toplevel_list_v1::Event::Finished => {
                Logger::log("Ext toplevel list finished");
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtForeignToplevelHandleV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &ExtForeignToplevelHandleV1,
        event: <ExtForeignToplevelHandleV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let obj_id = proxy.id().protocol_id();

        match event {
            ext_foreign_toplevel_handle_v1::Event::Title { title } => {
                if let Some(info) = state.toplevel_state.toplevels.get_mut(&obj_id) {
                    Logger::log(&format!("Ext title (id={}): {}", obj_id, title));
                    info.title = Some(title);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                if let Some(info) = state.toplevel_state.toplevels.get_mut(&obj_id) {
                    Logger::log(&format!("Ext app_id (id={}): {}", obj_id, app_id));
                    info.app_id = Some(app_id);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Done => {
                Logger::log(&format!("Ext done (id={})", obj_id));
                state.toplevel_state.check_focus();
            }
            ext_foreign_toplevel_handle_v1::Event::Closed => {
                Logger::log(&format!("Ext closed (id={})", obj_id));
                state.toplevel_state.toplevels.remove(&obj_id);
                state.toplevel_state.check_focus();
            }
            _ => {}
        }
    }
}

impl Dispatch<ZcosmicToplevelInfoV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ZcosmicToplevelInfoV1,
        event: <ZcosmicToplevelInfoV1 as Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zcosmic_toplevel_info_v1::Event::Done => {
                Logger::log("Cosmic toplevel info done");
                state.toplevel_state.check_focus();
            }
            zcosmic_toplevel_info_v1::Event::Finished => {
                Logger::log("Cosmic toplevel info finished");
            }
            _ => {}
        }
    }
}

impl Dispatch<ZcosmicToplevelHandleV1, CosmicHandleData> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ZcosmicToplevelHandleV1,
        event: <ZcosmicToplevelHandleV1 as Proxy>::Event,
        data: &CosmicHandleData,
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let ext_id = data.ext_id;

        match event {
            zcosmic_toplevel_handle_v1::Event::State { state: states } => {
                let activated = states
                    .chunks_exact(4)
                    .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .any(|s| s == STATE_ACTIVATED);

                Logger::log(&format!(
                    "Cosmic state (ext_id={}): activated={}",
                    ext_id, activated
                ));

                if let Some(info) = state.toplevel_state.toplevels.get_mut(&ext_id) {
                    info.activated = activated;
                }
            }
            zcosmic_toplevel_handle_v1::Event::Title { title } => {
                Logger::log(&format!("Cosmic title (ext_id={}): {}", ext_id, title));
                if let Some(info) = state.toplevel_state.toplevels.get_mut(&ext_id) {
                    info.title = Some(title);
                }
            }
            zcosmic_toplevel_handle_v1::Event::AppId { app_id } => {
                Logger::log(&format!("Cosmic app_id (ext_id={}): {}", ext_id, app_id));
                if let Some(info) = state.toplevel_state.toplevels.get_mut(&ext_id) {
                    info.app_id = Some(app_id);
                }
            }
            zcosmic_toplevel_handle_v1::Event::Done => {
                Logger::log(&format!("Cosmic done (ext_id={})", ext_id));
                state.toplevel_state.check_focus();
            }
            zcosmic_toplevel_handle_v1::Event::Closed => {
                Logger::log(&format!("Cosmic closed (ext_id={})", ext_id));
                state.toplevel_state.toplevels.remove(&ext_id);
                state.toplevel_state.ext_to_cosmic.remove(&ext_id);
                state.toplevel_state.check_focus();
            }
            _ => {}
        }
    }
}
