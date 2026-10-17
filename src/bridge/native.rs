#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
    }

    #[namespace = "native"]
    unsafe extern "C++" {
        include!("native.h");

        fn set_quick_style(style: &str);
        fn set_app_font(family: &str, point_size: f64);
        fn register_font_data(data: &[u8]) -> String;
        fn install_image_providers(engine: Pin<&mut QQmlApplicationEngine>);
        fn has_root_objects(engine: &QQmlApplicationEngine) -> bool;
        fn set_clipboard_uris(uris: &Vec<String>, cut: bool);
        fn clipboard_uris() -> Vec<String>;
        fn clipboard_is_cut() -> bool;
        fn mime_type(path: &str, by_content: bool) -> String;
        fn mime_comment(mime: &str) -> String;
    }
}
