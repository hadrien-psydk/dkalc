use text_canvas::TextCanvas;
use big_dec;
use big_dec::BigDec;
use token;
use token::Token;
use funcs;

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
	Bd(big_dec::Error),
	Fn(funcs::Error)
}

impl EvalError {
	fn to_string(&self) -> String {
		match *self {
			EvalError::Bd(ref nv_error) => nv_error.to_string(),
			EvalError::Fn(ref fn_error) => fn_error.to_string()
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
			Token::Add => match BigDec::add(val_left, val_right) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			},
			Token::Sub => match BigDec::sub(val_left, val_right) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			},
			Token::Mul => match BigDec::mul(val_left, val_right) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			},
			Token::Div => match BigDec::div(val_left, val_right) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			},
			Token::Mod => match BigDec::div_mod(val_left, val_right) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			},
			Token::Func(name) => match funcs::eval_func(name, val_left) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Fn(err))
			},
			Token::Fact => match BigDec::fact(val_left) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			},
			Token::And => match BigDec::and(val_left, val_right) {
				Ok(val) => Ok(val),
				Err(err) => Err(EvalError::Bd(err))
			}
		};
		nv_result
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

fn parse_subfactor_parenthesis(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
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

fn parse_subfactor_function(tg: &mut TokenGetter, arena: &mut TreeArena, name: token::Name) -> ParseResult {
	// Parenthesis expression
	let inside_id_res = parse_expression(tg, arena);
	let inside_id = match inside_id_res {
		ParseResult::None => {
			// Missing function argument
			return ParseResult::Fail(format!("missing function argument"));
		},
		ParseResult::Fail(err) => {
			return ParseResult::Fail(err)
		},
		ParseResult::Some(inside_id) => {
			// Ok, continue
			inside_id
		}
	};

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

	// Alloc a node to keep the function name and the subtree inside the parenthesis
	let (node, node_id) = arena.alloc_node(Token::Func(name));
	node.left_id = Some(inside_id);
	node.right_id = None;
	ParseResult::Some(node_id)
}

// S -> '-'? number
// S -> '(' X ')'
// S -> 'func(' X ')'
fn parse_subfactor(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
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
		Token::ParOpen => {
			return parse_subfactor_parenthesis(tg, arena);
		},
		Token::Func(name) => {
			return parse_subfactor_function(tg, arena, name);
		},
		_ => {
			return ParseResult::Fail(format!("unexpected {}", op.to_string()));
		},
	}
}

// F -> S '!'?
fn parse_factor(tg: &mut TokenGetter, arena: &mut TreeArena) -> ParseResult {
	let sf_id = match parse_subfactor(tg, arena) {
		ParseResult::None => { return ParseResult::None; },
		ParseResult::Fail(err_str) => { return ParseResult::Fail(err_str); }
		ParseResult::Some(id) => id
	};

	// Is there a '!' ?
	let op = match tg.peek() {
		Some(op) => op,
		None => { return ParseResult::Some(sf_id); }
	};

	match *op {
		Token::Fact => {
			tg.next();
		},
		_ => { return ParseResult::Some(sf_id); }
	}

	let (node, node_id) = arena.alloc_node(Token::Fact);
	node.left_id = Some(sf_id);
	node.right_id = None;
	ParseResult::Some(node_id)
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
		let (node, node_id) = arena.alloc_node((*op).clone());
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

		let (node, node_id) = arena.alloc_node((*op).clone());
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

#[allow(dead_code)]
pub fn eval_input(input: &str) -> String {
	eval_input_debug(input, false)
}

pub fn eval_input_debug(input: &str, debug: bool) -> String {
	let tokens_res = token::tokenize(input);
	let tokens = match tokens_res {
		Ok(tokens) => tokens,
		Err(err) => { return err.to_string(); }
	};

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

pub struct DetailedEval {
	pub state_str: String,
	pub result_dec: String,
	pub result_hex: String
}

pub fn eval_input_debug_detailed(input: &str, debug: bool) -> DetailedEval {
	let mut ret = DetailedEval {
		state_str: "".into(),
		result_dec: "--".into(),
		result_hex: "--".into()
	};
	let tokens_res = token::tokenize(input);
	let tokens = match tokens_res {
		Ok(tokens) => tokens,
		Err(err) => { ret.state_str = err.to_string(); return ret; }
	};

	match make_tree(tokens) {
		Ok(tree) => {
			if debug {
				println!("{}", tree.to_string());
			}
			match tree.eval() {
				Ok(nv) => {
					//println!("dbg: {:?}", nv);
					ret.result_dec = nv.to_string();
					ret.result_hex = nv.to_string_hex(8);
					ret
				},
				Err(err) => {
					ret.state_str = err.to_string(); ret
				}
			}
		},
		Err(err) => {
			ret.state_str = err.into(); ret
		}
	}
}

#[test]
fn test_eval() {
	assert_eq!("20", eval_input("8/2 + 1 + 3*5"));
	assert_eq!("24", eval_input("2*3*4"));
	assert_eq!("2", eval_input("7%5"));
	assert_eq!("-2", eval_input("-3+1"));
	assert_eq!("720", eval_input("6!"));
}

#[test]
fn test_order() {
	assert_eq!("0", eval_input("4+2-3-3"));
}
