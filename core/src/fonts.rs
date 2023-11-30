// use crate::imports::*;

pub fn init_fonts(cc: &eframe::CreationContext<'_>) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Light);

    // ---
    fonts.font_data.insert(
        "ubuntu_mono".to_owned(),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSans-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
        egui::FontData::from_static(include_bytes!(
            "../../resources/fonts/UbuntuMono/UbuntuMono-Regular.ttf"
        )),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Light.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Light.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Light.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "ubuntu_mono".to_owned());

    // ---
    // fonts.font_data.insert(
    //     "noto_sans_extra_light".to_owned(),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSans-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
    //     egui::FontData::from_static(include_bytes!(
    //         "../../resources/fonts/NotoSans-ExtraLight.ttf"
    //     )),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Light.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Light.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Light.ttf")),
    // );

    // fonts
    //     .families
    //     // .entry(egui::FontFamily::Proportional)
    //     .entry(egui::FontFamily::Name("noto_sans_extra_light".into()))
    //     .or_default()
    //     .insert(0, "noto_sans_extra_light".to_owned());

    // // ---

    // ---
    // fonts.font_data.insert(
    //     "noto_sans_mono_extra_condensed".to_owned(),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSans-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
    //     egui::FontData::from_static(include_bytes!(
    //         "../../resources/fonts/NotoSans/NotoSansMono_ExtraCondensed-Light.ttf"
    //     )),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Light.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Light.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Regular.ttf")),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Light.ttf")),
    // );

    // fonts
    //     .families
    //     // .entry(egui::FontFamily::Proportional)
    //     .entry(egui::FontFamily::Name("noto_sans_mono_extra_condensed".into()))
    //     .or_default()
    //     .insert(0, "noto_sans_mono_extra_condensed".to_owned());

    // // ---

    // ---
    fonts.font_data.insert(
        "noto_sans_mono_light".to_owned(),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSans-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
        egui::FontData::from_static(include_bytes!(
            "../../resources/fonts/NotoSans/NotoSansMono-Light.ttf"
        )),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSansMono-Light.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/SourceCodePro-Light.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../../resources/fonts/RobotoMono-Light.ttf")),
    );

    fonts
        .families
        // .entry(egui::FontFamily::Proportional)
        .entry(egui::FontFamily::Name("noto_sans_mono_light".into()))
        .or_default()
        .insert(0, "noto_sans_mono_light".to_owned());

    // ---

    // #[cfg(windows)]
    // {
    //     let font_file = {
    //         // c:/Windows/Fonts/msyh.ttc
    //         let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok()?);
    //         font_path.push("Fonts");
    //         font_path.push("msyh.ttc");
    //         font_path.to_str()?.to_string().replace("\\", "/")
    //     };
    //     Some(font_file)
    // }

    // "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"
    // "/System/Library/Fonts/Hiragino Sans GB.ttc"

    // ---
    // fonts.font_data.insert(
    //     "test_font".to_owned(),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSans-Regular.ttf")),
    //     egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
    // );

    // fonts
    //     .families
    //     .entry(egui::FontFamily::Proportional)
    //     .or_default()
    //     .insert(0, "test_font".to_owned());

    // ---

    // #[cfg(target_os = "macos")]
    // if let Ok(font) = std::fs::read("/System/Library/Fonts/Hiragino Sans GB.ttc") {

    //     fonts.font_data.insert(
    //         "hiragino-sans-gb".to_owned(),
    //         // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
    //         egui::FontData::from_owned(font),
    //     );

    //     fonts
    //         .families
    //         .entry(egui::FontFamily::Proportional)
    //         .or_default()
    //         // .insert(0, "hiragino".to_owned());
    //         .push("hiragino-sans-gb".to_owned());
    // }

    #[cfg(target_os = "linux")]
    if let Ok(font) = std::fs::read("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc") {
        fonts.font_data.insert(
            "noto-sans-cjk".to_owned(),
            // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
            egui::FontData::from_owned(font),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            // .insert(0, "hiragino".to_owned());
            .push("noto-sans-cjk".to_owned());
    }

    cc.egui_ctx.set_fonts(fonts);
}

// fn _init_fonts(&self, egui_ctx: &egui::Context) {
//     let mut fonts = egui::FontDefinitions::default();

//     // Install my own font (maybe supporting non-latin characters).
//     // .ttf and .otf files supported.
//     fonts.font_data.insert(
//         "my_font".to_owned(),
//         egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
//     );

//     // Put my font first (highest priority) for proportional text:
//     fonts
//         .families
//         .entry(egui::FontFamily::Proportional)
//         .or_default()
//         .insert(0, "open_sans".to_owned());
//     // .insert(0, "my_font".to_owned());

//     // Put my font as last fallback for monospace:
//     // fonts
//     //     .families
//     //     .entry(egui::FontFamily::Monospace)
//     //     .or_default()
//     //     .push("my_font".to_owned());

//     // Tell egui to use these fonts:
//     egui_ctx.set_fonts(fonts);
// }
