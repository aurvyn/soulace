use eframe::egui;

fn main() -> eframe::Result {
	let mut souls = 0;
	let mut options = eframe::NativeOptions::default();
	options.renderer = eframe::Renderer::Wgpu;
	eframe::run_simple_native("Soulace", options, move |ctx, _frame| {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Welcome to the Soulace browser!");
			ui.label("Soulace is a browser that uses Pug/Sass/Nim instead of HTML/CSS/JS.");
			ui.horizontal(|ui| {
				ui.label("It is a work in progress, and not all features are implemented yet.");
				ui.label("This is just a test page.");
			});
			ui.label(format!("You have collected {} souls!", souls));
			if ui.button("collect soul").clicked() {
				souls += 1;
			}
		});
	})
}
