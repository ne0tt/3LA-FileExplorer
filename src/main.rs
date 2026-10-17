mod apps;
mod archive;
mod bridge;
mod crypt;
mod config;
mod fsutil;
mod icons;
mod smb;
mod smb_io;
mod trashdir;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

use bridge::native::ffi as native;

fn main() {
    config::migrate();
    native::set_quick_style("Basic");
    let mut app = QGuiApplication::new();
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_organization_name(&QString::from("3LA"));
        app.as_mut().set_application_name(&QString::from("FileExplorer"));
        app.as_mut().set_application_display_name(&QString::from("3LA File Explorer"));
        QGuiApplication::set_desktop_file_name(&QString::from("3LA-FileExplorer"));
    }
    native::set_app_font("JetBrainsMono Nerd Font", 10.0);
    // Dialog display face (titles, progress figures), carried over from SPX-FileExplorer.
    native::register_font_data(include_bytes!("../fonts/ChakraPetch-SemiBold.ttf"));
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        native::install_image_providers(engine.as_mut());
        engine.load(&QUrl::from("qrc:/qt/qml/ThreeLA/FileExplorer/qml/Main.qml"));
    }
    // A QML error leaves no window; don't linger as an invisible process.
    if !engine.as_ref().is_some_and(native::has_root_objects) {
        eprintln!("3la-fileexplorer: failed to load the user interface");
        std::process::exit(1);
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
