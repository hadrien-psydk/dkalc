extern crate gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Entry, Label, Box, Orientation, Menu, MenuBar, MenuItem,
	AboutDialog, License, CssProvider, StyleContext};

mod text_canvas;
mod eval;
mod big_dec;
mod token;
mod funcs;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct AppArgs {
	debug_mode: bool,
	expression: String
}

fn parse_app_args() -> AppArgs {
	let args: Vec<String> = std::env::args().collect();
	let app_args = {
		if args.len() <= 1 {
			AppArgs { debug_mode: false, expression: String::new() }
		}
		else if args[1] == "--debug" {
			if args.len() <= 3 {
				AppArgs { debug_mode: true, expression: args[2].clone() }
			}
			else {
				AppArgs { debug_mode: true, expression: String::new() }
			}
		}
		else {
			AppArgs { debug_mode: false, expression: args[1].clone() }
		}
	};
	app_args
}

fn main() {
	let app_args = parse_app_args();

	//println!("{}", eval::eval_input("4+2-3-3"));
	/*
	println!("{}", big_dec::BigDec::div(
		big_dec::BigDec::from_i32(100),
		big_dec::BigDec::from_i32(48)
		).to_string()
	);

	println!("{}", big_dec::BigDec::div_mod(
		big_dec::BigDec::from_i32(100),
		big_dec::BigDec::from_i32(48)
		).to_string()
	);*/


	if gtk::init().is_err() {
		println!("Failed to initialize GTK.");
		return;
	}

	let window = Window::new(WindowType::Toplevel);
	window.set_title("dkalc");
	window.set_wmclass("dkalc", "dkalc");
	window.set_default_size(350, 100);

	let gtk_box = Box::new(Orientation::Vertical, 3);
	window.add(&gtk_box);

	let file_menu = Menu::new();
	/*
	let settings_menu_item = MenuItem::new_with_label("Settings");
	file_menu.append(&settings_menu_item);
	let help_menu_item = MenuItem::new_with_label("Help");
	file_menu.append(&help_menu_item);
	*/
	let about_menu_item = MenuItem::new_with_label("About");
	file_menu.append(&about_menu_item);
	let quit_menu_item = MenuItem::new_with_label("Quit");
	file_menu.append(&quit_menu_item);

	let file_menu_item = MenuItem::new_with_label("File");
	file_menu_item.set_submenu(Some(&file_menu));

	let menu_bar = MenuBar::new();
	menu_bar.append(&file_menu_item);

	gtk_box.pack_start(&menu_bar, true, true, 0);

	let label = Label::new(Some("0"));
	label.set_name("result");
	gtk_box.pack_start(&label, true, true, 0);

	let entry = Entry::new();
	gtk_box.pack_start(&entry, true, true, 0);

	let css_provider = CssProvider::new();
	let css = "#result { font-family: monospace; font-size: 15px; }";
	if let Err(err) = css_provider.load_from_data(css) {
		println!("css_provider.load_from_data failed: {}", err);
		return;
	}
	StyleContext::add_provider_for_screen(
		&gtk::WidgetExt::get_screen(&window).unwrap(),
		&css_provider,
		800 // gtk_sys::GTK_STYLE_PROVIDER_PRIORITY_USER
		);

	window.connect_delete_event(|_, _| {
		gtk::main_quit();
		Inhibit(false)
	});
	window.show_all();

	about_menu_item.connect_activate(move |_| {
		let ad = AboutDialog::new();
        ad.set_authors(&["Hadrien Nilsson"]);
        ad.set_website_label(Some("psydk.org"));
        ad.set_website(Some("http://psydk.org"));
        ad.set_title("About dkalc");
		ad.set_program_name("dkalc");
		ad.set_version(Some(VERSION));
		ad.set_license_type(License::Gpl20);
        ad.set_transient_for(Some(&window));
        ad.run();
		ad.destroy();
	});

	quit_menu_item.connect_activate(|_| {
		gtk::main_quit();
	});

	entry.set_text(&app_args.expression);
	entry.set_position(-1);
	let result = eval::eval_input_debug(&app_args.expression, app_args.debug_mode);
	label.set_label(&result);

	entry.connect_changed(move |arg| {
		if let Some(str) = arg.get_chars(0, -1) {
			let result = eval::eval_input_debug(&str, app_args.debug_mode);
			label.set_label(&result);
		}
	});

	gtk::main();
}

