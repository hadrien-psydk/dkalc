use text_canvas::TextCanvas;
use num_val;
use num_val::BigDec;
use token;
use token::Token;

struct Node {
    token: Token,
    left_id: Option<usize>,
    right_id: Option<usize>
}

struct TreeArena {
	nodes: Vec<Node>,
}

impl TreeArena {
	fn new_with_size(size: usize) -> TreeArena {
		TreeArena { nodes: Vec::with_capacity(size) }
	}
	fn alloc_node(&mut self, token: Token) -> (&mut Node, usize) {
		self.nodes.push( Node { token: token, left_id: None, right_id: None } );
		let id = self.nodes.len() - 1;
		(&mut self.nodes[id], id)
	}
	fn alloc_leaf(&mut self, token: Token) -> usize {
		self.nodes.push( Node { token: token, left_id: None, right_id: None } );
		self.nodes.len() - 1
	}
}

enum EvalError {
	Nv(num_val::Error)
}

impl EvalError {
	fn to_string(&self) -> String {
		match *self {
			EvalError::Nv(ref nv_error) => nv_error.to_string()
		}
	}
}

struct Tree {
    arena: TreeArena,
	root_id: usize,
}

impl Tree {
	fn get_node(&self, id: usize) -> &Node {
		&self.arena.nodes[id]
	}

	fn draw_node(&self, node_id: usize, pad: usize, canvas: &mut TextCanvas) {
		let node = self.get_node(node_id);

		canvas.do_str_fix(&node.token.to_string());
		canvas.down();

		if node.left_id.is_some() {
			let state0 = canvas.get_state();
			canvas.do_str("\u{2534}");
			canvas.left(pad/2+2);
			
			let state1 = canvas.get_state();
			canvas.do_str("\u{250c}");
			canvas.do_str_n("\u{2500}", pad/2);
			canvas.right(1);
			canvas.set_state(state1);

			canvas.down();
			self.draw_node(node.left_id.unwrap(), pad/2-1, canvas);
			canvas.set_state(state0);
		}

		if node.right_id.is_some() {
			let state0 = canvas.get_state();
			canvas.do_str("\u{2534}");

			canvas.do_str_n("\u{2500}", pad/2);
			canvas.do_str_fix("\u{2510}");
			canvas.down();
			self.draw_node(node.right_id.unwrap(), pad/2, canvas);
			canvas.set_state(state0);
		}
	}

	fn to_string(&self) -> String {
		let mut canvas = TextCanvas::new(64, 64);
		let pad = 16;
		canvas.right(pad);
		self.draw_node(self.root_id, pad, &mut canvas);
		canvas.to_string()
	}

	fn eval_node(&self, node_id: usize) -> Result<BigDec, EvalError> {
		let node = self.get_node(node_id);
		
		let val_left = if let Some(left_id) = node.left_id {
			try!(self.eval_node(left_id))
		}
		else {
			BigDec::zero()
		};

		let val_right = if let Some(right_id) = node.right_id {
			try!(self.eval_node(right_id))
		}
		else {
			BigDec::zero()
		};

		let nv_result = match node.token {
			Token::Nothing => Ok(BigDec::zero()),
			Token::Number(ref nv) => Ok(*nv),
			Token::ParOpen => Ok(BigDec::zero()),
			Token::ParClose => Ok(BigDec::zero()),
			Token::Add => BigDec::add(val_left, val_right),
			Token::Sub => BigDec::sub(val_left, val_right),
			Token::Mul => BigDec::mul(val_left, val_right),
			Token::Div => BigDec::div(val_left, val_right),
			Token::Mod => BigDec::div_mod(val_left, val_right),
		};
		if nv_result.is_err() {
			return Err(EvalError::Nv(nv_result.unwrap_err()));
		}
		Ok(nv_result.unwrap())
	}

	fn eval(&self) -> Result<BigDec, EvalError> {
		self.eval_node(self.root_id)
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
				None => { return ParseResult::Fail("missing number".into()); }
			};
			match *op_next {
				Token::Number(nv) => {
					let node_id = arena.alloc_leaf(Token::Number(nv.negate()));
					return ParseResult::Some(node_id);
				},
				_ => {
					return ParseResult::Fail(format!(
						"expected number instead of {}", op_next.to_string()));
				}
			}
		}
		Token::Number(nv) => {
			let node_id = arena.alloc_leaf(Token::Number(nv));
			return ParseResult::Some(node_id);
		},
		Token::ParOpen => (),
		_ => {
			return ParseResult::Fail(format!("unexpected {}", op.to_string()));
		}
	}

	// Parenthesis expression
	let inside = parse_expression(tg, arena);
	
	// We expect the closing parenthesis
	let op2 = match tg.next() {
		Some(op2) => op2,
		None => {
			return ParseResult::Fail(format!("missing ')'"));
		}
	};

	match *op2 {
		Token::ParClose => (),
		_ => {
			return ParseResult::Fail(format!("expected ')', found: {}", op2.to_string()));
		}
	}
	return inside;
}

// { * F }*
// { / F }*
// { % F }*
fn parse_term_right(tg: &mut TokenGetter, arena: &mut TreeArena, mut root_id: usize) -> ParseResult {
	loop {
		let op = match tg.peek() {
			Some(op) => op,
			None => { break; }
		};

		match *op {
			Token::Mul | Token::Div | Token::Mod => (),
			_ => { break; }
		}
		tg.next();

		let right_id = match parse_factor(tg, arena) {
			ParseResult::None => return ParseResult::Fail("missing factor".into()),
			ParseResult::Fail(err) => return ParseResult::Fail(err),
			ParseResult::Some(right_id) => right_id
		};
		let (mut node, node_id) = arena.alloc_node((*op).clone());
		node.left_id = Some(root_id);
		node.right_id = Some(right_id);
		root_id = node_id;
	}
	ParseResult::Some(root_id)
}

// T -> F { * F }*
// T -> F { / F }*
// T -> F { % F }*
fn parse_term(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	// F
	match parse_factor(tg, arena) {
		ParseResult::None => ParseResult::None,
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(root_id) =>  {
			// { * F }
			match parse_term_right(tg, arena, root_id) {
				ParseResult::None => ParseResult::Some(root_id),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(root_id) =>  {
					ParseResult::Some(root_id)
				}
			}
		}
	}
}

// { - T }*
// { + T }*
fn parse_expression_right(tg: &mut TokenGetter, arena: &mut TreeArena, mut root_id: usize) -> ParseResult {
	loop {
		let op = match tg.peek() {
			Some(op) => op,
			None => { break; }
		};

		match *op {
			Token::Add | Token::Sub => (),
			_ => { break; }
		}
		tg.next();

		let right_id = match parse_term(tg, arena) {
			ParseResult::None => return ParseResult::Fail("missing term".into()),
			ParseResult::Fail(err) => return ParseResult::Fail(err),
			ParseResult::Some(right_id) => right_id
		};

		let (mut node, node_id) = arena.alloc_node((*op).clone());
		node.left_id = Some(root_id);
		node.right_id = Some(right_id);
		root_id = node_id;
	}
	ParseResult::Some(root_id)
}

// X -> T { - T }*
// X -> T { + T }*
fn parse_expression(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	// T
	match parse_term(tg, arena) {
		ParseResult::None => ParseResult::None,
		ParseResult::Fail(err) => ParseResult::Fail(err),
		ParseResult::Some(root_id) => { 
			// { - T }
			match parse_expression_right(tg, arena, root_id) {
				ParseResult::None => ParseResult::Some(root_id),
				ParseResult::Fail(err) => ParseResult::Fail(err),
				ParseResult::Some(root_id) => {
					ParseResult::Some(root_id)
				}
			}
		}
	}
}

// creates the evaluation tree from the list of tokens
fn make_tree(mut tokens: Vec<Token>) -> Result<Tree, String> {
	let mut arena = TreeArena::new_with_size(tokens.len());
	let mut tg = TokenGetter { tokens: &mut tokens, index: 0 };
	let root_id = match parse_expression(&mut tg, &mut arena) {
		ParseResult::None => arena.alloc_leaf(Token::Nothing),
		ParseResult::Some(root_id) => root_id,
		ParseResult::Fail(err) => return Err(err),
	};
	let tree = Tree { arena: arena, root_id: root_id };
	Ok(tree)
}

pub fn eval_input(input: &str) -> String {
	eval_input_debug(input, false)
}

pub fn eval_input_debug(input: &str, debug: bool) -> String {
	let tokens_res = token::tokenize(input);
	let tokens = match tokens_res {
		Ok(tokens) => tokens,
		Err(err) => { return err.to_string(); }
	};
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
			if debug {
				println!("{}", tree.to_string());
			}
			match tree.eval() {
				Ok(nv) => {
					//println!("dbg: {:?}", nv);
					nv.to_string()
				},
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

#[test]
fn test_order() {
	assert_eq!("0", eval_input("4+2-3-3"));
}
