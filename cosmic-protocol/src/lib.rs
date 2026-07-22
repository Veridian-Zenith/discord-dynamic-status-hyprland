pub mod zcosmic_toplevel_info {
    use wayland_client;
    use wayland_client::protocol::wl_output;
    use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        use wayland_protocols::ext::foreign_toplevel_list::v1::client::__interfaces::*;
        wayland_scanner::generate_interfaces!(
            "../cosmic/protocols/cosmic-toplevel-info-unstable-v1.xml"
        );
    }

    use self::__interfaces::*;

    wayland_scanner::generate_client_code!(
        "../cosmic/protocols/cosmic-toplevel-info-unstable-v1.xml"
    );
}
