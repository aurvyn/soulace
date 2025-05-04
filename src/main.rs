use std::fs;
use eframe::egui::{self, RichText};
use serde_yaml::Mapping;

enum Heading {
	Plain,
	H1,
	H2,
	H3,
	H4,
	H5,
	H6,
}

enum Element {
	Label(String, Heading),
	Button(String),
	Code(String),
	TextEdit(String),
	CodeEdit(String),
	CheckBox(bool, String),
	Link(String, String),
	Toggle(bool, String),
	Details(String, Vec<Element>),
	Unknown,
}

impl Element {
	fn new(mapping: &Mapping) -> Self {
		if let Some((tag, value)) = mapping.iter().next() {
			let value = value.as_str().unwrap_or_default().to_string();
			match tag.as_str().unwrap_or_default() {
				"p" => Element::Label(value, Heading::Plain),
				"h1" => Element::Label(value, Heading::H1),
				"h2" => Element::Label(value, Heading::H2),
				"h3" => Element::Label(value, Heading::H3),
				"h4" => Element::Label(value, Heading::H4),
				"h5" => Element::Label(value, Heading::H5),
				"h6" => Element::Label(value, Heading::H6),
				"button" => Element::Button(value),
				"code" => Element::Code(value),
				"textedit" => Element::TextEdit(value),
				"codeedit" => Element::CodeEdit(value),
				"checkbox" => Element::CheckBox(false, value),
				"link" => Element::Link(value, String::new()),
				"toggle" => Element::Toggle(false, value),
				"details" => {
					let header = mapping
						.get("header")
						.and_then(|text| text.as_str())
						.unwrap_or_default()
						.to_string();
					let empty_vec = vec![];
					let summary = mapping
						.get("summary")
						.and_then(|val| val.as_sequence())
						.unwrap_or(&empty_vec)
						.iter()
						.filter_map(|item| item.as_mapping())
						.map(Element::new)
						.collect();
					Element::Details(header, summary)
				}
				_ => Element::Unknown,
			}
		} else {
			Element::Unknown
		}
	}
}

const FILE_PATH: &str = "test/home.yaml";

fn parse_yaml() -> (String, Vec<Element>) {
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
	let body = doc.get("body")
		.expect("Failed to get 'body' from YAML")
		.as_sequence()
		.expect("Failed to parse 'body' as sequence")
		.iter()
		.map(|item| item.as_mapping().map_or(Element::Unknown, Element::new))
		.collect();
	(title, body)
}

fn draw_elements(ui: &mut egui::Ui, body: &mut Vec<Element>) {
	for element in body {
		match element {
			Element::Label(text, heading) => {
				let text = RichText::new(&*text);
				ui.label(match heading {
					Heading::Plain => text.size(16.0),
					Heading::H1 => text.size(32.0).strong(),
					Heading::H2 => text.size(26.0).strong(),
					Heading::H3 => text.size(18.72).strong(),
					Heading::H4 => text.size(16.0).strong(),
					Heading::H5 => text.size(13.28).strong(),
					Heading::H6 => text.size(10.72).strong(),
				});
			}
			Element::Button(text) => {
				ui.button(&*text).clicked();
			}
			Element::Code(text) => {
				ui.code(text);
			}
			Element::TextEdit(text) => {
				ui.text_edit_singleline(text);
			}
			Element::CodeEdit(text) => {
				ui.code_editor(text);
			}
			Element::CheckBox(checked, text) => {
				ui.checkbox(checked, &*text);
			}
			Element::Link(text, _url) => {
				ui.link(&*text).clicked();
			}
			Element::Toggle(selected, text) => {
				ui.toggle_value(selected, &*text);
			}
			Element::Details(header, summary) => {
				ui.collapsing(&*header, |ui| {
					draw_elements(ui, summary);
				});
			}
			_ => {
				ui.label("Unknown element");
			}
		}
	}
}

fn main() -> eframe::Result {
	let (title, mut body) = parse_yaml();
	let mut options = eframe::NativeOptions::default();
	options.renderer = eframe::Renderer::Wgpu;
	eframe::run_simple_native(&title, options, move |ctx, _frame| {
		egui::CentralPanel::default().show(ctx, |ui| {
			draw_elements(ui, &mut body);
		});
	})
}
