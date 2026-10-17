use std::fs;

use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

const SINGLETONS: &[&str] = &["Theme.qml", "G.qml"];

fn main() {
    let mut qml: Vec<String> = fs::read_dir("qml")
        .expect("qml directory")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".qml"))
        .collect();
    qml.sort();
    let files = qml.iter().map(|n| {
        println!("cargo::rerun-if-changed=qml/{n}");
        QmlFile::from(format!("qml/{n}")).singleton(SINGLETONS.contains(&n.as_str()))
    });
    println!("cargo::rerun-if-changed=qml");
    println!("cargo::rerun-if-changed=cpp/native.h");

    CxxQtBuilder::new_qml_module(QmlModule::new("ThreeLA.FileExplorer").qml_files(files))
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .include_dir("cpp")
        .cpp_file("cpp/native.cpp")
        .files([
            "src/bridge/native.rs",
            "src/bridge/folder_model.rs",
            "src/bridge/file_ops.rs",
            "src/bridge/system.rs",
        ])
        .build();
}
