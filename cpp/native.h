#pragma once

#include "rust/cxx.h"

class QQmlApplicationEngine;

namespace native {

void set_quick_style(rust::Str style);
void set_app_font(rust::Str family, double point_size);
rust::String register_font_data(rust::Slice<const uint8_t> data);
void install_image_providers(QQmlApplicationEngine& engine);
bool has_root_objects(const QQmlApplicationEngine& engine);
void set_clipboard_uris(const rust::Vec<rust::String>& uris, bool cut);
rust::Vec<rust::String> clipboard_uris();
bool clipboard_is_cut();
rust::String mime_type(rust::Str path, bool by_content);
rust::String mime_comment(rust::Str mime);

}
