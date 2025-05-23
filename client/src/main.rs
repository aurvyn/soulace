use eframe::egui::{self, Color32, Context, RichText, TextStyle, ViewportCommand};
use mlua::Lua;
use reqwest::Client;
use simplecss::StyleSheet;
use pollster::FutureExt;

use soulace::{
	Element,
	Heading,
	Styles,
};

const SERVER_URL: &str = "http://localhost:3030";
const WEBSITE: &str = "test/home.yaml";
const DEFAULT_STYLE: &str = "template/default.sass";

async fn parse_yaml(url: &str, yaml_code: String, client: &Client) -> (String, Vec<Element>, Styles, String) {
	if yaml_code.is_empty() || !url.ends_with(".yaml") {
		return ("Erm what?".to_string(), vec![
			Element::Label("Hmm. We're having trouble finding that site.".to_string(), Heading::H1),
			Element::Label("We can't connect to that server bruh".to_string(), Heading::H3),
		], Styles::default(), String::new());
	}
	let website = url.split('/').next().unwrap();
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
		let sass = fetch_file(client, &format!("{website}/{path}")).await;
		grass::from_string(sass, &grass::Options::default().input_syntax(grass::InputSyntax::Sass)).unwrap()
	} else {
		grass::from_path(DEFAULT_STYLE, &grass::Options::default()).unwrap()
	};
	let styles = parse_css(css);
	let script = if let Some(path) = head.get("script").and_then(|script| script.as_str()) {
		fetch_file(client, &format!("{website}/{path}")).await
	} else {
		String::new()
	};
	let body = doc.get("body")
		.expect("Failed to get 'body' from YAML")
		.as_sequence()
		.expect("Failed to parse 'body' as sequence")
		.iter()
		.map(|item| item.as_mapping().map_or(Element::Unknown, Element::new))
		.collect();
	(title, body, styles, script)
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

fn draw_elements(ui: &mut egui::Ui, body: &mut Vec<Element>, styles: &Styles, lua: &Lua) {
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
			Element::TextEdit(text, onsubmit) => {
				let response = ui.add(egui::TextEdit::singleline(text));
				if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
					let script = format!("{onsubmit}(\"{text}\")");
					lua.load(script).exec().expect("Failed to execute Lua script");
				}
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
					draw_elements(ui, summary, styles, lua);
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

async fn fetch_site(ctx: &Context, client: &Client, url: &str) -> (Vec<Element>, Styles, String) {
	let response = fetch_file(client, url).await;
	let (title, body, styles, script) = parse_yaml(url, response, &client).await;
	ctx.send_viewport_cmd(ViewportCommand::Title(title));
	(body, styles, script)
}

#[tokio::main]
async fn main() -> eframe::Result {
	let mut page_promise: Option<poll_promise::Promise<(Vec<Element>, Styles, String)>> = None;
	let lua = Lua::new();
	let client = Client::new();
	let response = fetch_file(&client, WEBSITE).block_on();
	let (title, mut body, mut styles, script) = parse_yaml(WEBSITE, response, &client).block_on();
	lua.load(script).exec().expect("Failed to execute Lua script");
	let mut url = WEBSITE.to_string();
	let mut current_url = url.clone();
	let mut url_history = vec![];
	let mut url_future = vec![];
	let mut options = eframe::NativeOptions::default();
	options.renderer = eframe::Renderer::Wgpu;
	eframe::run_simple_native(&title, options, move |ctx, _frame| {
		if let Some(p) = &page_promise {
			if let Some((elements, style, script)) = p.ready() {
				body = elements.clone();
				styles = style.clone();
				lua.load(script).exec().expect("Failed to execute Lua script");
				page_promise = None;
			}
		}
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			ui.horizontal_centered(|ui| {
				if ui.button("⚙").clicked() {
					println!("Settings clicked");
				}
				if ui.add_enabled(!url_history.is_empty(), egui::Button::new("⬅")).clicked() {
					url = url_history.pop().unwrap();
					url_future.push(current_url.clone());
					current_url = url.clone();
					let ctx = ctx.clone();
					let client = client.clone();
					let url = url.clone();
					page_promise = Some(poll_promise::Promise::spawn_async(async move {
						fetch_site(&ctx, &client, &url).await
					}));
				}
				if ui.add_enabled(!url_future.is_empty(), egui::Button::new("➡")).clicked() {
					url = url_future.pop().unwrap();
					url_history.push(current_url.clone());
					current_url = url.clone();
					let ctx = ctx.clone();
					let client = client.clone();
					let url = url.clone();
					page_promise = Some(poll_promise::Promise::spawn_async(async move {
						fetch_site(&ctx, &client, &url).await
					}));
				}
				if ui.button("⟳").clicked() {
					let ctx = ctx.clone();
					let client = client.clone();
					let url = url.clone();
					page_promise = Some(poll_promise::Promise::spawn_async(async move {
						fetch_site(&ctx, &client, &url).await
					}));
				}
				let response = ui.add(egui::TextEdit::singleline(&mut url).hint_text("Enter URL").font(TextStyle::Heading));
				if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && url != current_url {
					url_history.push(current_url.clone());
					url_future.clear();
					current_url = url.clone();
					let ctx = ctx.clone();
					let client = client.clone();
					let url = url.clone();
					page_promise = Some(poll_promise::Promise::spawn_async(async move {
						fetch_site(&ctx, &client, &url).await
					}));
				}
			});
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			draw_elements(ui, &mut body, &styles, &lua);
		});
	})
}
