use eframe::{
    egui::{Context, FontData, FontDefinitions},
    epaint::FontFamily,
};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};

use crate::setting::Setting;

fn load_fonts(fonts: &mut FontDefinitions, handle: Handle, name: String) {
    let buf: Vec<u8> = match handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => std::fs::read(path).unwrap(),
    };
    fonts
        .font_data
        .insert(name.clone(), FontData::from_owned(buf));
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push(name.clone());
    }
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(name.clone());
    }
}

pub fn load_system_font(ctx: &Context, setting: &Setting) {
    let font_family = SystemSource::new();
    let property = Properties::new();
    let fonts: Vec<_> = setting
        .get_fonts()
        .into_iter()
        .map(|s| ([FamilyName::Title(s.clone())], s))
        .filter_map(|(fm, s)| {
            font_family
                .select_best_match(&fm, &property)
                .ok()
                .and_then(|f| Some((f, s)))
        })
        .collect();
    println!("Font Family List {:#?}", fonts);
    let mut font = FontDefinitions::default();
    for (handle, name) in fonts {
        load_fonts(&mut font, handle, name);
    }
    ctx.set_fonts(font);
}
