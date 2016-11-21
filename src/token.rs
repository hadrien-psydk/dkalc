use std;
use num_val;
use num_val::NumVal;

#[allow(dead_code)]
#[derive(Copy,Clone)]
pub enum Token {
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
	pub fn to_string(&self) -> std::borrow::Cow<'static, str> {
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

pub enum Error {
	Nothing, // End of string
	BadChar(char),
	BadNum(num_val::Error)
}

impl Error {
	pub fn to_string(&self) -> String {
		match *self {
			Error::Nothing => "".into(),
			Error::BadChar(c) => format!("bad char: '{}'", c),
			Error::BadNum(ref nverr) => nverr.to_string()
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
	
	fn next_token(&mut self) -> Result<Token, Error> {
		let ret;
		loop {
			let num_res = NumVal::parse_chars(&mut self.input_chars);
			match num_res {
				Ok(num) => {
					ret = Ok(Token::Number(num));
					break;
				},
				Err(err) => {
					match err {
						num_val::Error::ParseNothing => (), // Not a problem
						_ => { return Err(Error::BadNum(err)); }
					}
				}
			}

			let c_opt = self.input_chars.next();
			if c_opt.is_none() {
				ret = Err(Error::Nothing);
				break;
			}
			let c = c_opt.unwrap();
			if c == '(' {
				ret = Ok(Token::ParOpen);
				break;
			}
			else if c == ')' {
				ret = Ok(Token::ParClose);
				break;
			}
			else if c == '+' {
				ret = Ok(Token::Add);
				break;
			}
			else if c == '-' {
				ret = Ok(Token::Sub);
				break;
			}
			else if c == '*' {
				ret = Ok(Token::Mul);
				break;
			}
			else if c == '/' {
				ret = Ok(Token::Div);
				break;
			}
			else if c == '%' {
				ret = Ok(Token::Mod);
				break;
			}
			else if c == ' ' {
				// continue
			}
			else
			{
				return Err(Error::BadChar(c));
			}
		}
		ret
	}
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Error> {
	let mut ret = Vec::new();
	let mut context = InputContext::new(input);
	loop {
		let token_res = context.next_token();
		match token_res {
			Ok(token) => ret.push(token),
			Err(err) => {
				match err {
					Error::Nothing => (),
					_ => { return Err(err); }
				}
				break;
			}
		}
	}
	Ok(ret)
}
