use eframe::egui::Color32;
use serde_yaml::Mapping;

pub enum Heading {
	Plain,
	H1,
	H2,
	H3,
	H4,
	H5,
	H6,
}

pub enum Element {
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
	pub fn new(mapping: &Mapping) -> Self {
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

pub struct Styles {
	pub body: ContainerStyle,
	pub p: TextStyle,
	pub h1: TextStyle,
	pub h2: TextStyle,
	pub h3: TextStyle,
	pub h4: TextStyle,
	pub h5: TextStyle,
	pub h6: TextStyle,
	pub button: ContainerStyle,
	pub code: TextStyle,
	pub link: TextStyle,
	pub details: ContainerStyle,
}

impl Styles {
	pub fn get_mut(&mut self, tag: &str) -> Option<&mut dyn Style> {
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

pub trait Style {
	fn set_background_color(&mut self, color: Color32);
	fn set_color(&mut self, color: Color32);
	fn set_font_size(&mut self, size: f32);
}

pub struct ContainerStyle {
	pub background_color: Color32,
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

pub struct TextStyle {
	pub color: Color32,
	pub font_size: f32,
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
