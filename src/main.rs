extern crate gtk;
//extern crate gdk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Entry, Label, Box, Orientation, Menu, MenuBar, MenuItem,
	AboutDialog, License, CssProvider, StyleContext};

#[derive(Copy,Clone)]
struct NumVal {
	val: i32
}

impl NumVal {
	fn to_string(&self) -> String {
		format!("{}", self.val)
	}

	fn is_zero(&self) -> bool {
		self.val == 0
	}

	fn zero() -> NumVal {
		NumVal { val: 0 }
	}

	fn add(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val + nv1.val }
	}

	fn sub(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val - nv1.val }
	}

	fn mul(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val * nv1.val }
	}

	fn div(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val / nv1.val }
	}

	fn div_mod(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val % nv1.val }
	}
}

#[allow(dead_code)]
#[derive(Copy,Clone)]
enum Token {
	Nothing,
	Number(NumVal),
	ParOpen,
	ParClose,
	Add,
	Sub,
	Mul,
	Div,
	Mod
}


impl Token {
	fn to_string(&self) -> std::borrow::Cow<'static, str> {
		match *self {
			Token::Nothing => "_".into(),
			Token::Number(ref nv) => nv.to_string().into(),
			Token::ParOpen => "(".into(),
			Token::ParClose => ")".into(),
			Token::Add => "+".into(),
			Token::Sub => "-".into(),
			Token::Mul => "*".into(),
			Token::Div => "/".into(),
			Token::Mod => "%".into(),
		}
	}
}


struct InputContext<'a> {
	input_chars: std::iter::Peekable<std::str::Chars<'a>>
}

impl<'a> InputContext<'a> {
	fn new(input: &str) -> InputContext {
		let ic = input.chars().peekable();
		InputContext { input_chars: ic }
	}
	
	fn parse_number(&mut self, c_first: char) -> Option<NumVal> {
		let mut val = 0;
		val += c_first.to_digit(10).unwrap() as i32;
		loop {
			let val2 = {
				let c_opt = self.input_chars.peek();
				if c_opt.is_none() {
					None
				}
				else {
					let c = c_opt.unwrap();
					if c.is_digit(10) {
						Some(c.to_digit(10).unwrap() as i32)
					}
					else {
						None
					}
				}
			};
			if val2.is_none() {
				break;
			}
			val *= 10;
			val += val2.unwrap();
			self.input_chars.next();
		}
		Some( NumVal { val: val } )
	}

	fn next_token(&mut self) -> Option<Token> {
		let ret;
		loop {
			let c_opt = self.input_chars.next();
			if c_opt.is_none() {
				ret = None;
				break;
			}
			let c = c_opt.unwrap();
			if c.is_digit(10) {
				let num_opt = self.parse_number(c);
				if num_opt.is_none() {
					ret = None;
					break;
				}
				ret = Some(Token::Number(num_opt.unwrap()));
				break;
			}
			else if c == '(' {
				ret = Some(Token::ParOpen);
				break;
			}
			else if c == ')' {
				ret = Some(Token::ParClose);
				break;
			}
			else if c == '+' {
				ret = Some(Token::Add);
				break;
			}
			else if c == '-' {
				ret = Some(Token::Sub);
				break;
			}
			else if c == '*' {
				ret = Some(Token::Mul);
				break;
			}
			else if c == '/' {
				ret = Some(Token::Div);
				break;
			}
			else if c == '%' {
				ret = Some(Token::Mod);
				break;
			}
			else if c == ' ' {
				// continue
			}
		}
		ret
	}
}

fn tokenize(input: &str) -> Vec<Token> {
	let mut ret = Vec::new();
	let mut context = InputContext::new(input);
	loop {
		let token_opt = context.next_token();
		match token_opt {
			Some(token) => ret.push(token),
			None => break
		}
	}
	ret
}

struct Node {
    token: Token,
    left: Option<usize>,
    right: Option<usize>
}

struct TreeArena {
	arena: Vec<Node>,
}

impl TreeArena {
	fn new_with_size(size: usize) -> TreeArena {
		TreeArena { arena: Vec::with_capacity(size) }
	}
	fn push_single(&mut self, token: Token) -> usize {
		self.arena.push( Node { token: token, left: None, right: None } );
		self.arena.len() - 1
	}
	fn push_dual(&mut self, token: Token, left: usize, right: usize) -> usize {
		self.arena.push( Node { token: token, left: Some(left), right: Some(right) } );
		self.arena.len() - 1
	}
}


const AC_WIDTH: usize = 64;
const AC_HEIGHT: usize = 64;
struct AsciiCanvas {
	text: [[char; AC_WIDTH]; AC_HEIGHT],
	x: usize,
	y: usize,
	y_max: usize,
}
struct AsciiCanvasState {
	x: usize,
	y: usize,
}

impl AsciiCanvas {
	fn new() -> AsciiCanvas {
		AsciiCanvas { text: [[' '; AC_WIDTH];AC_HEIGHT], x: 0, y: 0, y_max: 0 }
	}
	fn down(&mut self) {
		self.y += 1;
		if self.y > self.y_max {
			self.y_max = self.y;
		}
	}
	fn left(&mut self, len: usize) {
		self.x -= len;
	}
	fn right(&mut self, len: usize) {
		self.x += len;
	}
	fn do_str(&mut self, s: &str) {
		let chars = s.chars();
		for c in chars {
			self.text[self.y][self.x] = c;
			self.x += 1;
		}
	}
	fn do_str_n(&mut self, s: &str, n: usize) {
		for _ in 0..n {
			self.do_str(s);
		}
	}
	fn do_str_fix(&mut self, s: &str) {
		let chars = s.chars();
		let mut i = 0;
		for c in chars {
			self.text[self.y][self.x + i] = c;
			i += 1;
		}
	}
	fn to_string(&self) -> String {
		let mut ret = String::new();
		for i in 0..self.y_max+1 {
			for j in 0..AC_WIDTH {
				ret.push(self.text[i][j]);
			}
			ret.push('\n');
		}
		ret
	}
	fn get_state(&self) -> AsciiCanvasState {
		AsciiCanvasState { x: self.x, y: self.y }
	}
	fn set_state(&mut self, state: AsciiCanvasState) {
		self.x = state.x;
		self.y = state.y;
	}
}

enum EvalError {
	DivideByZero
}

impl EvalError {
	fn to_string(&self) -> String {
		match *self {
			EvalError::DivideByZero => "divide by zero".into()
		}
	}
}

struct Tree {
    arena: TreeArena,
	root: usize,
}

impl Tree {
	fn get_node(&self, id: usize) -> &Node {
		&self.arena.arena[id]
	}

	fn draw_node(&self, node_id: usize, pad: usize, canvas: &mut AsciiCanvas) {
		let node = self.get_node(node_id);

		canvas.do_str_fix(&node.token.to_string());
		canvas.down();

		if node.left.is_some() {
			let state0 = canvas.get_state();
			canvas.do_str("\u{2534}");
			canvas.left(pad/2+2);
			
			let state1 = canvas.get_state();
			canvas.do_str("\u{250c}");
			canvas.do_str_n("\u{2500}", pad/2);
			canvas.right(1);
			canvas.set_state(state1);

			canvas.down();
			self.draw_node(node.left.unwrap(), pad/2-1, canvas);
			canvas.set_state(state0);
		}

		if node.right.is_some() {
			let state0 = canvas.get_state();
			canvas.do_str("\u{2534}");

			canvas.do_str_n("\u{2500}", pad/2);
			canvas.do_str_fix("\u{2510}");
			canvas.down();
			self.draw_node(node.right.unwrap(), pad/2, canvas);
			canvas.set_state(state0);
		}
	}

	fn to_string(&self) -> String {
		let mut canvas = AsciiCanvas::new();
		let pad = 16;
		canvas.right(pad);
		self.draw_node(self.root, pad, &mut canvas);
		canvas.to_string()
	}

	fn eval_node(&self, node_id: usize) -> Result<NumVal, EvalError> {
		let node = self.get_node(node_id);
		
		let val_left = if let Some(left) = node.left {
			try!(self.eval_node(left))
		}
		else {
			NumVal::zero()
		};

		let val_right = if let Some(right) = node.right {
			try!(self.eval_node(right))
		}
		else {
			NumVal::zero()
		};

		let nv_result = match node.token {
			Token::Nothing => NumVal::zero(),
			Token::Number(ref nv) => *nv,
			Token::ParOpen => NumVal::zero(),
			Token::ParClose => NumVal::zero(),
			Token::Add => NumVal::add(val_left, val_right),
			Token::Sub => NumVal::sub(val_left, val_right),
			Token::Mul => NumVal::mul(val_left, val_right),
			Token::Div => {
				if val_right.is_zero() {
					return Err(EvalError::DivideByZero)
				}
				else {
					NumVal::div(val_left, val_right)
				}
			},
			Token::Mod => {
				if val_right.is_zero() {
					return Err(EvalError::DivideByZero)
				}
				else {
					NumVal::div_mod(val_left, val_right)
				}
			},
		};
		Ok(nv_result)
	}

	fn eval(&self) -> Result<NumVal, EvalError> {
		self.eval_node(self.root)
	}
}

struct TokenGetter<'a> {
	tokens: &'a Vec<Token>,
	index: usize,
}

impl<'a> TokenGetter<'a> {
	fn peek(&mut self) -> Option<&'a Token> {
		self.tokens.get(self.index)
	}

	fn next(&mut self) -> Option<&'a Token> {
		if let Some(ret) = self.tokens.get(self.index) {
			self.index += 1;
			Some(ret)
		}
		else {
			None
		}
	}
}

enum ParseResult {
	None,
	Fail(String),
	Some(usize),
}

// F -> number
// F -> '(' X ')'
fn parse_factor(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	let op = match tg.next() {
		Some(op) => op,
		None => { return ParseResult::None; }
	};

	match *op {
		Token::Number(_) => {
			let opc = (*op).clone();
			let node_id = arena.push_single(opc);
			return ParseResult::Some(node_id);
		},
		Token::ParOpen => (),
		_ => {
			return ParseResult::Fail(format!("parse_factor error: found {}", op.to_string()));
		}
	}
	let inside = parse_expression(tg, arena);
	
	// We expect the closing parenthesis
	let op2 = match tg.next() {
		Some(op2) => op2,
		None => {
			return ParseResult::Fail(format!("parse_factor error: missing ')'"));
		}
	};

	match *op2 {
		Token::ParClose => (),
		_ => {
			return ParseResult::Fail(format!("parse_factor error: expected ')', found: {}", op2.to_string()));
		}
	}
	return inside;
}

// { * F }*
// { / F }*
// { % F }*
fn parse_term_right(tg: &mut TokenGetter, arena: &mut TreeArena, left: usize) -> ParseResult {
	let op = match tg.peek() {
		Some(op) => op,
		None => { return ParseResult::None; }
	};

	match *op {
		Token::Mul | Token::Div | Token::Mod => (),
		_ => {
			return ParseResult::None;
		}
	}
	tg.next();

	match parse_factor(tg, arena) {
		ParseResult::None => ParseResult::Fail("parse_term_right: missing factor".into()),
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(right) => {
			let right2_pr = match parse_term_right(tg, arena, right) {
				ParseResult::None => ParseResult::Some(right),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(right2) => ParseResult::Some(right2)
			};

			if let ParseResult::Some(right2) = right2_pr {
				let op2 = (*op).clone();
				let node_id = arena.push_dual(op2, left, right2);
				ParseResult::Some(node_id)
			}
			else {
				right2_pr
			}
		}
	}
}

// T -> F { * F }*
// T -> F { / F }*
// T -> F { % F }*
fn parse_term(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	// F
	match parse_factor(tg, arena) {
		ParseResult::None => ParseResult::None,
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(left) =>  {
			// { * F }
			match parse_term_right(tg, arena, left) {
				ParseResult::None => ParseResult::Some(left),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(right) =>  {
					ParseResult::Some(right)
				}
			}
		}
	}
}

// { - T }*
// { + T }*
fn parse_expression_right(tg: &mut TokenGetter, arena: &mut TreeArena, left: usize) -> ParseResult {
	let op = match tg.peek() {
		Some(op) => op,
		None => { return ParseResult::None; }
	};

	//println!("parse_expression_right: matching {}", op.to_string());

	match *op {
		Token::Add | Token::Sub => (),
		_ => {
			return ParseResult::None;
		}
	}
	tg.next();

	match parse_term(tg, arena) {
		ParseResult::None => ParseResult::Fail("parse_expression_right: missing term".into()),
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(right) => {
			let right2_pr = match parse_expression_right(tg, arena, right) {
				ParseResult::None => ParseResult::Some(right),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(right2) => ParseResult::Some(right2)
			};

			if let ParseResult::Some(right2) = right2_pr {
				let op2 = (*op).clone();
				let node_id = arena.push_dual(op2, left, right2);
				ParseResult::Some(node_id)
			}
			else {
				right2_pr
			}
		}
	}
}

// X -> T { - T }*
// X -> T { + T }*
fn parse_expression(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	// T
	match parse_term(tg, arena) {
		ParseResult::None => ParseResult::None,
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(left) => { 
			// { - T }
			match parse_expression_right(tg, arena, left) {
				ParseResult::None => ParseResult::Some(left),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(right) => {
					ParseResult::Some(right)
				}
			}
		}
	}
}

fn make_tree(mut tokens: Vec<Token>) -> Result<Tree, String> {
	let mut arena = TreeArena::new_with_size(tokens.len());
	let mut tg = TokenGetter { tokens: &mut tokens, index: 0 };
	let root = match parse_expression(&mut tg, &mut arena) {
		ParseResult::None => arena.push_single(Token::Nothing),
		ParseResult::Some(root) => root,
		ParseResult::Fail(err) => return Err(err),
	};
	let tree = Tree { arena: arena, root: root };
	Ok(tree)
}

fn eval_input(input: &str) -> String {
	let tokens = tokenize(input);
	/*
	print!("{} tokens: ", tokens.len());
	for t in &tokens {
		print!("[{}] ", t.to_string());
	}
	println!("");
	*/

	/*
	println!("tree:");
	*/

	match make_tree(tokens) {
		Ok(tree) => {
			println!("{}", tree.to_string());
			match tree.eval() {
				Ok(nv) => nv.to_string(),
				Err(err) => err.to_string()
			}
		},
		Err(err) => err.into()
	}
}

#[test]
fn test_eval() {
	assert_eq!("20", eval_input("8/2 + 1 + 3*5"));
	assert_eq!("24", eval_input("2*3*4"));
	assert_eq!("2", eval_input("7%5"));
}

fn main() {
	//println!("= {}", eval_input("3%2"));

	if gtk::init().is_err() {
		println!("Failed to initialize GTK.");
		return;
	}

	let window = Window::new(WindowType::Toplevel);
	window.set_title("dkalc");
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
	gtk_box.pack_start(&label, true, true, 0);

	let entry = Entry::new();
	gtk_box.pack_start(&entry, true, true, 0);

	let css_provider = CssProvider::new();
	let css = "label { font: monospace 15; }";
	if let Err(err) = css_provider.load_from_data(css) {
		println!("{}", err);
		return;
	}
	StyleContext::add_provider_for_screen(
		//&gdk::Screen::get_default().unwrap(),
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
		ad.set_version(Some("1.0-beta"));
		ad.set_license_type(License::Gpl20);
        ad.set_transient_for(Some(&window));
        ad.run();
		ad.destroy();
	});

	quit_menu_item.connect_activate(|_| {
		gtk::main_quit();
	});

	entry.connect_changed(move |arg| {
		if let Some(str) = arg.get_chars(0, -1) {
			let result = eval_input(&str);
			//println!("text changed: {} = {}", str, result);
			label.set_label(&result);
		}
	});
	
	gtk::main();
}

