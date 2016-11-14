use std;
use text_canvas::TextCanvas;
use num_val::NumVal;

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
				let num_opt = NumVal::parse(&mut self.input_chars, c);
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
	nodes: Vec<Node>,
}

impl TreeArena {
	fn new_with_size(size: usize) -> TreeArena {
		TreeArena { nodes: Vec::with_capacity(size) }
	}
	fn push_single(&mut self, token: Token) -> usize {
		self.nodes.push( Node { token: token, left: None, right: None } );
		self.nodes.len() - 1
	}
	fn push_dual(&mut self, token: Token, left: usize, right: usize) -> usize {
		self.nodes.push( Node { token: token, left: Some(left), right: Some(right) } );
		self.nodes.len() - 1
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
		&self.arena.nodes[id]
	}

	fn draw_node(&self, node_id: usize, pad: usize, canvas: &mut TextCanvas) {
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
		let mut canvas = TextCanvas::new(64, 64);
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

// F -> '-'? number
// F -> '(' X ')'
fn parse_factor(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	let op = match tg.next() {
		Some(op) => op,
		None => { return ParseResult::None; }
	};

	match *op {
		Token::Sub => {
			// Wants number
			let op_next = match tg.next() {
				Some(op_next) => op_next,
				None => { return ParseResult::Fail("parse_factor error: missing number".into()); }
			};
			match *op_next {
				Token::Number(nv) => {
					let node_id = arena.push_single(Token::Number(nv.negate()));
					return ParseResult::Some(node_id);
				},
				_ => {
					return ParseResult::Fail(format!(
						"parse_factor error: expected number instead of {}", op_next.to_string()));
				}
			}
		}
		Token::Number(nv) => {
			let node_id = arena.push_single(Token::Number(nv));
			return ParseResult::Some(node_id);
		},
		Token::ParOpen => (),
		_ => {
			return ParseResult::Fail(format!("parse_factor error: found {}", op.to_string()));
		}
	}

	// Parenthesis expression
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

pub fn eval_input(input: &str) -> String {
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
	assert_eq!("-2", eval_input("-3+1"));
}
