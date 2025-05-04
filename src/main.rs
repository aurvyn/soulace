use std::fs;
use eframe::egui::{self, RichText};

const FILE_PATH: &str = "test/home.yaml";

fn main() -> eframe::Result {
	let yaml_code = fs::read_to_string(FILE_PATH).expect("Something went wrong reading the file");
	let yaml = serde_yaml::from_str::<serde_yaml::Value>(&yaml_code).expect("Failed to parse YAML");
	let doc = yaml
		.as_mapping()
		.expect("Failed to parse YAML as mapping");
	let head = doc
		.get("head")
		.expect("Failed to get 'head' from YAML")
		.as_mapping()
		.expect("Failed to parse 'head' as mapping");
	let title = head
		.get("title")
		.expect("Failed to get 'title' from 'head'")
		.as_str()
		.expect("Failed to parse 'title' as text")
		.to_string();
	let body = doc
		.get("body")
		.expect("Failed to get 'body' from YAML")
		.as_sequence()
		.expect("Failed to parse 'body' as sequence")
		.clone();

	let mut souls = 0;
	let mut options = eframe::NativeOptions::default();
	options.renderer = eframe::Renderer::Wgpu;
	eframe::run_simple_native(&title, options, move |ctx, _frame| {
		egui::CentralPanel::default().show(ctx, |ui| {
			for item in &body {
				item.as_mapping().map(|mapping| {
					let (tag, value) = mapping.iter().next().unwrap();
					let tag = tag.as_str().expect("Failed to parse tag as valid syntax");
					let value = value.as_str().expect("Failed to parse value as text");
					let text = RichText::new(value);
					ui.label(match tag {
						"p" => text,
						"h1" => text.size(30.0).strong(),
						"h2" => text.size(28.0).strong(),
						"h3" => text.size(26.0).strong(),
						"h4" => text.size(24.0).strong(),
						"h5" => text.size(22.0).strong(),
						"h6" => text.size(20.0).strong(),
						_ => panic!("Unknown tag: {}", tag),
					})
				}).unwrap_or_else(|| {
					ui.label("Invalid item in YAML")
				});
			}
			ui.label(format!("You have collected {} souls!", souls));
			if ui.button("collect soul").clicked() {
				souls += 1;
			}
		});
	})
}
