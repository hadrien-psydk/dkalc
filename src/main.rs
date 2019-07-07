extern crate gtk;
extern crate gio;

use gtk::prelude::*;
use gio::prelude::*;

use gio::MenuExt;
use gio::SimpleAction;
//use gio::ActionMapExt;

use gtk::{ApplicationWindow};

/*
//use gio::*;*/

//use glib::{self, Variant};
use std::env::args;

mod text_canvas;
mod eval;
mod big_dec;
mod token;
mod funcs;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct AppArgs {
	debug_mode: bool,
	expression: String
}

fn parse_app_args<T: AsRef<str>>(args: Vec<T>) -> AppArgs {
	if args.len() <= 1 {
		AppArgs { debug_mode: false, expression: String::new() }
	}
	else if args[1].as_ref() == "--debug" {
		if args.len() > 2 {
			AppArgs { debug_mode: true, expression: args[2].as_ref().to_string() }
		}
		else {
			AppArgs { debug_mode: true, expression: String::new() }
		}
	}
	else {
		AppArgs { debug_mode: false, expression: args[1].as_ref().to_string() }
	}
}

#[test]
fn test_parse_app_args() {
	let app_args0 = parse_app_args(vec!["dkalc", ""]);
	assert_eq!(app_args0.debug_mode, false);
	assert_eq!(app_args0.expression, "");

	let app_args1 = parse_app_args(vec!["dkalc", "42"]);
	assert_eq!(app_args1.debug_mode, false);
	assert_eq!(app_args1.expression, "42");

	let app_args2 = parse_app_args(vec!["dkalc", "--debug"]);
	assert_eq!(app_args2.debug_mode, true);
	assert_eq!(app_args2.expression, "");

	let app_args3 = parse_app_args(vec!["dkalc", "--debug", "42"]);
	assert_eq!(app_args3.debug_mode, true);
	assert_eq!(app_args3.expression, "42");
}

struct Header {
	pub header_bar: gtk::HeaderBar,
	pub hamburger_button: gtk::MenuButton
}

impl Header {
	fn new() -> Header {
		let header_bar = gtk::HeaderBar::new();
		header_bar.set_title("Dkalc");
		header_bar.set_show_close_button(true);

		let hamburger_button = gtk::MenuButton::new();
		let hamburger_image = gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Button.into());
		hamburger_button.set_image(&hamburger_image);
		header_bar.pack_end(&hamburger_button);

		let menu = gio::Menu::new();
		//menu.append("Preferences", "win.preferences");
	  	menu.append("About", "win.about");

		let popover = gtk::Popover::new_from_model(&hamburger_button, &menu);
		hamburger_button.set_popover(&popover);

		Header { header_bar, hamburger_button }
	}
}

/*
fn add_action<M, A>(map: &M, name: &str, action: A)
	where M: ActionMapExt,
		for<'r, 's> A: Fn(&'r gio::SimpleAction, &'s Option<glib::Variant>) + 'static
{
	let sa = SimpleAction::new("about", None);
	map.add_action(&sa);
	sa.connect_activate(action);
}*/

fn show_about(window: &ApplicationWindow) {
	let ad = gtk::AboutDialog::new();
	ad.set_authors(&["Hadrien Nilsson"]);
	ad.set_website_label(Some("psydk.org"));
	ad.set_website(Some("http://psydk.org"));
	ad.set_title("About dkalc");
	ad.set_program_name("dkalc");
	ad.set_version(Some(VERSION));
	ad.set_license_type(gtk::License::Gpl20);
	ad.set_transient_for(Some(window));
	ad.set_logo_icon_name("dkalc");
	ad.run();
	ad.destroy();
}

fn build_ui(app: &gtk::Application, app_args: &AppArgs) {
	let window = ApplicationWindow::new(app);

	let header = Header::new();
	window.set_titlebar(&header.header_bar);
	window.set_wmclass("dkalc", "Dkalc");
	window.set_default_size(350, 100);

	////////////////////////////////////////////////////////////////
	let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 3);
	window.add(&gtk_box);

	// Result display
	let label_state = gtk::Label::new(Some(""));
	gtk::WidgetExt::set_name(&label_state, "state");
	gtk_box.pack_start(&label_state, true, true, 0);

	let label_result_dec = gtk::Label::new(Some("0"));
	gtk::WidgetExt::set_name(&label_result_dec, "result");
	gtk_box.pack_start(&label_result_dec, true, true, 0);

	let label_result_hex = gtk::Label::new(Some("0x0"));
	gtk::WidgetExt::set_name(&label_result_hex, "result");
	gtk_box.pack_start(&label_result_hex, true, true, 0);

	// CSS
	let css_provider = gtk::CssProvider::new();
	let css = "#state { color: #800; } #result { font-family: monospace; font-size: 15px; }";
	if let Err(err) = css_provider.load_from_data(css.as_bytes()) {
		println!("css_provider.load_from_data failed: {}", err);
		return;
	}
	gtk::StyleContext::add_provider_for_screen(
		&gtk::WidgetExt::get_screen(&window).unwrap(),
		&css_provider,
		800 // gtk_sys::GTK_STYLE_PROVIDER_PRIORITY_USER
		);

	// Input box
	let entry = gtk::Entry::new();
	gtk_box.pack_start(&entry, true, true, 0);

	entry.connect_changed({
		let debug_mode = app_args.debug_mode;
		move |arg| {
			if let Some(str) = arg.get_chars(0, -1) {
				let de = eval::eval_input_debug_detailed(&str, debug_mode);
				label_state.set_label(&de.state_str);
				label_result_dec.set_label(&de.result_dec);
				label_result_hex.set_label(&de.result_hex);
			}
		}
	});

	entry.set_text(&app_args.expression);

	window.show_all();

	let sa = SimpleAction::new("about", None);
	window.add_action(&sa);
	sa.connect_activate(move |_, _| { show_about(&window); });
}

fn main() {
	let app_args = parse_app_args(std::env::args().collect());

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

	let app = gtk::Application::new("psydk.dkalc",
		gio::ApplicationFlags::HANDLES_COMMAND_LINE)
			.expect("Application::new failed");
	app.connect_startup(move |arg| build_ui(arg, &app_args));

	app.connect_activate(|_| {}); // Make GTK happy
	app.connect_command_line(|_, _| { 1 }); // Make GTK happy
	app.run(&args().collect::<Vec<_>>());
}
