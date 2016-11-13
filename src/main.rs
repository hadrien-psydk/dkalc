extern crate gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Entry, Label, Box, Orientation, Menu, MenuBar, MenuItem};
use std::slice;

#[derive(Copy,Clone)]
struct NumVal {
	val: i32
}

impl NumVal {
	fn to_string(&self) -> String {
		format!("{}", self.val)
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
	fn set_left(&mut self, left: usize, right: usize) {
		self.arena[right].left = Some(left);
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
		for c in chars {
			self.text[self.y][self.x] = c;
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

	fn eval_node(&self, node_id: usize) -> i32 {
		let node = self.get_node(node_id);
		
		let val_left = if let Some(left) = node.left {
			self.eval_node(left)
		}
		else {
			0
		};

		let val_right = if let Some(right) = node.right {
			self.eval_node(right)
		}
		else {
			0
		};

		match node.token {
			Token::Nothing => 0,
			Token::Number(ref nv) => nv.val,
			Token::ParOpen => 0,
			Token::ParClose => 0,
			Token::Add => val_left + val_right,
			Token::Sub => val_left - val_right,
			Token::Mul => val_left * val_right,
			Token::Div => val_left / val_right,
		}
	}
	fn eval(&self) -> NumVal {
		let val = self.eval_node(self.root);
		NumVal { val: val }
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
fn parse_term_right(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	let op = match tg.peek() {
		Some(op) => op,
		None => { return ParseResult::None; }
	};

	match *op {
		Token::Mul | Token::Div => (),
		_ => {
			return ParseResult::None;
		}
	}
	tg.next();

	match parse_factor(tg, arena) {
		ParseResult::None => ParseResult::Fail("parse_term_right: missing factor".into()),
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(right) => {
			let op2 = (*op).clone();
			let node_id = arena.push_dual(op2, 0usize, right);
			ParseResult::Some(node_id)
		}
	}
}

// T -> F { * F }*
// T -> F { / F }*
fn parse_term(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	// F
	match parse_factor(tg, arena) {
		ParseResult::None => ParseResult::None,
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(left) =>  {
			// { * F }
			match parse_term_right(tg, arena) {
				ParseResult::None => ParseResult::Some(left),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(right) =>  {
					arena.set_left(left, right);
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

fn make_tree(mut tokens: Vec<Token>) -> Option<Tree> {
	let mut arena = TreeArena::new_with_size(tokens.len());
	let mut tg = TokenGetter { tokens: &mut tokens, index: 0 };
	let root = match parse_expression(&mut tg, &mut arena) {
		ParseResult::None => arena.push_single(Token::Nothing),
		ParseResult::Some(root) => root,
		ParseResult::Fail(err) => {
			println!("make_tree error: {}", err);
			return None;
		}
	};
	let tree = Tree { arena: arena, root: root };
	Some(tree)
}

fn eval_input(input: &str) -> String {
	let tokens = tokenize(input);
	print!("{} tokens: ", tokens.len());
	for t in &tokens {
		print!("[{}] ", t.to_string());
	}
	println!("");

	println!("tree:");
	if let Some(tree) = make_tree(tokens) {
		println!("{}", tree.to_string());
		tree.eval().to_string()
	}
	else {
		"<no tree>".into()
	}
}

fn main() {
	println!("= {}", eval_input("8/2 + 1 + 3*5"));
	/*
	if gtk::init().is_err() {
		println!("Failed to initialize GTK.");
		return;
	}

	let window = Window::new(WindowType::Toplevel);
	window.set_title("dkalc");
	window.set_default_size(350, 100);

	let gtk_box = Box::new(Orientation::Vertical, 6);
	window.add(&gtk_box);

	let file_menu = Menu::new();
	let settings_menu_item = MenuItem::new_with_label("Settings");
	file_menu.append(&settings_menu_item);
	let help_menu_item = MenuItem::new_with_label("Help");
	file_menu.append(&help_menu_item);
	let about_menu_item = MenuItem::new_with_label("About");
	file_menu.append(&about_menu_item);
	let quit_item = MenuItem::new_with_label("Quit");
	file_menu.append(&quit_item);

	let file_menu_item = MenuItem::new_with_label("File");
	file_menu_item.set_submenu(Some(&file_menu));

	let menu_bar = MenuBar::new();
	menu_bar.append(&file_menu_item);

	gtk_box.pack_start(&menu_bar, true, true, 0);

	let label = Label::new(Some("0"));
	gtk_box.pack_start(&label, true, true, 0);

	let entry = Entry::new();
	gtk_box.pack_start(&entry, true, true, 0);

	window.connect_delete_event(|_, _| {
		gtk::main_quit();
		Inhibit(false)
	});

	entry.connect_changed(|arg| {
		if let Some(str) = arg.get_chars(0, -1) {
			let result = eval_input(&str);
			println!("text changed: {} = {}", str, result);
		}
	});

	window.show_all();
	gtk::main();*/
}

