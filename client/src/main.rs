use std::vec;

use eframe::{egui::{self, Color32, RichText}};
use serde_yaml::Mapping;
use reqwest::Client;
use simplecss::StyleSheet;

const SERVER_URL: &str = "http://localhost:3030";
const WEBSITE: &str = "test";
const DEFAULT_STYLE: &str = "template/default.sass";

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

struct Styles {
	body: ContainerStyle,
	p: TextStyle,
	h1: TextStyle,
	h2: TextStyle,
	h3: TextStyle,
	h4: TextStyle,
	h5: TextStyle,
	h6: TextStyle,
	button: ContainerStyle,
	code: TextStyle,
	link: TextStyle,
	details: ContainerStyle,
}

impl Styles {
	fn get_mut(&mut self, tag: &str) -> Option<&mut dyn Style> {
		match tag {
			"body" => Some(&mut self.body),
			"p" => Some(&mut self.p),
			"h1" => Some(&mut self.h1),
			"h2" => Some(&mut self.h2),
			"h3" => Some(&mut self.h3),
			"h4" => Some(&mut self.h4),
			"h5" => Some(&mut self.h5),
			"h6" => Some(&mut self.h6),
			"button" => Some(&mut self.button),
			"code" => Some(&mut self.code),
			"link" => Some(&mut self.link),
			"details" => Some(&mut self.details),
			_ => None,
		}
	}
}

impl Default for Styles {
	fn default() -> Self {
		Styles {
			body: ContainerStyle {
				background_color: Color32::DARK_GRAY,
			},
			p: TextStyle {
				color: Color32::GRAY,
				font_size: 16.0,
			},
			h1: TextStyle {
				color: Color32::WHITE,
				font_size: 32.0,
			},
			h2: TextStyle {
				color: Color32::WHITE,
				font_size: 26.0,
			},
			h3: TextStyle {
				color: Color32::WHITE,
				font_size: 18.72,
			},
			h4: TextStyle {
				color: Color32::WHITE,
				font_size: 16.0,
			},
			h5: TextStyle {
				color: Color32::WHITE,
				font_size: 13.28,
			},
			h6: TextStyle {
				color: Color32::WHITE,
				font_size: 10.72,
			},
			button: ContainerStyle::default(),
			code: TextStyle::default(),
			link: TextStyle::default(),
			details: ContainerStyle::default(),
		}
	}
}

trait Style {
	fn set_background_color(&mut self, color: Color32);
	fn set_color(&mut self, color: Color32);
	fn set_font_size(&mut self, size: f32);
}

struct ContainerStyle {
	background_color: Color32,
}

impl Default for ContainerStyle {
	fn default() -> Self {
		Self {
			background_color: Color32::DARK_GRAY,
		}
	}
}

impl Style for ContainerStyle {
	fn set_background_color(&mut self, color: Color32) {
		self.background_color = color;
	}

	fn set_color(&mut self, _color: Color32) {}

	fn set_font_size(&mut self, _size: f32) {}
}

struct TextStyle {
	color: Color32,
	font_size: f32,
}

impl Default for TextStyle {
	fn default() -> Self {
		Self {
			color: Color32::LIGHT_GRAY,
			font_size: 16.0,
		}
	}
}

impl Style for TextStyle {
	fn set_background_color(&mut self, _color: Color32) {}

	fn set_color(&mut self, color: Color32) {
		self.color = color;
	}

	fn set_font_size(&mut self, size: f32) {
		self.font_size = size;
	}
}

async fn parse_yaml(yaml_code: String, client: Client) -> (String, Vec<Element>, Styles) {
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
	let css = if let Some(path) = head.get("style").and_then(|style| style.as_str()) {
		let sass = fetch_file(&client, &format!("{WEBSITE}/{path}")).await;
		grass::from_string(sass, &grass::Options::default().input_syntax(grass::InputSyntax::Sass)).unwrap()
	} else {
		grass::from_path(DEFAULT_STYLE, &grass::Options::default()).unwrap()
	};
	let styles = parse_css(css);
	let body = doc.get("body")
		.expect("Failed to get 'body' from YAML")
		.as_sequence()
		.expect("Failed to parse 'body' as sequence")
		.iter()
		.map(|item| item.as_mapping().map_or(Element::Unknown, Element::new))
		.collect();
	(title, body, styles)
}

fn parse_css(css_code: String) -> Styles {
	let mut styles = Styles::default();
	for rule in StyleSheet::parse(&css_code).rules {
		let Some(style) = styles.get_mut(&rule.selector.to_string()) else {
			continue;
		};
		for property in rule.declarations {
			match property.name {
				"background-color" => {
					if let Ok(color) = Color32::from_hex(property.value) {
						style.set_background_color(color);
					}
				}
				"color" => {
					if let Ok(color) = Color32::from_hex(property.value) {
						style.set_color(color);
					}
				}
				"font-size" => {
					if let Ok(size) = property.value.parse() {
						style.set_font_size(size);
					}
				}
				_ => ()
			}
		}
	};
	styles
}

fn draw_elements(ui: &mut egui::Ui, body: &mut Vec<Element>, styles: &Styles) {
	for element in body {
		match element {
			Element::Label(text, heading) => {
				let text = RichText::new(&*text);
				ui.label(match heading {
					Heading::Plain => text.size(styles.p.font_size).color(styles.p.color),
					Heading::H1 => text.size(styles.h1.font_size).color(styles.h1.color),
					Heading::H2 => text.size(styles.h2.font_size).color(styles.h2.color),
					Heading::H3 => text.size(styles.h3.font_size).color(styles.h3.color),
					Heading::H4 => text.size(styles.h4.font_size).color(styles.h4.color),
					Heading::H5 => text.size(styles.h5.font_size).color(styles.h5.color),
					Heading::H6 => text.size(styles.h6.font_size).color(styles.h6.color),
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
					draw_elements(ui, summary, styles);
				});
			}
			_ => {
				ui.label("Unknown element");
			}
		}
	}
}

async fn fetch_file(client: &Client, url: &str) -> String {
	client.get(format!("{SERVER_URL}/{url}"))
		.send().await
		.expect("Failed to send request")
		.text().await
		.expect("Failed to read response")
}

#[tokio::main]
async fn main() -> eframe::Result {
	let client = Client::new();
	let response = fetch_file(&client, &format!("{WEBSITE}/home.yaml")).await;
	let (title, mut body, styles) = parse_yaml(response, client).await;
	let mut options = eframe::NativeOptions::default();
	options.renderer = eframe::Renderer::Wgpu;
	eframe::run_simple_native(&title, options, move |ctx, _frame| {
		egui::CentralPanel::default().show(ctx, |ui| {
			draw_elements(ui, &mut body, &styles);
		});
	})
}
