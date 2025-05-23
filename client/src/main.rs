use eframe::{egui::{self, Color32, RichText}};
use reqwest::Client;
use simplecss::StyleSheet;

use soulace::{
	Element,
	Heading,
	Styles,
};

const SERVER_URL: &str = "http://localhost:3030";
const WEBSITE: &str = "test";
const DEFAULT_STYLE: &str = "template/default.sass";

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
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			ui.label(RichText::new("Top Panel").color(Color32::WHITE));
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			draw_elements(ui, &mut body, &styles);
		});
	})
}
