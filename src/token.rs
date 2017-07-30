use std;
use big_dec;
use big_dec::BigDec;

const MAX_NAME_LEN: u8 = 12;

// Stores a small function name
#[derive(Copy,Clone)]
pub struct Name {
	len: u8,
	bytes: [u8;MAX_NAME_LEN as usize]
}

impl Name {
	pub fn to_string(&self) -> String {
		let slice = &self.bytes[0..self.len as usize];
		let s = std::str::from_utf8(slice).unwrap();
		String::from(s)
	}
}

#[allow(dead_code)]
#[derive(Copy,Clone)]
pub enum Token {
	Nothing,
	Number(BigDec),
	ParOpen,
	ParClose,
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Func(Name), // Ends with a ParClose
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
			Token::Func(ref name) => {
				let mut name_par = name.to_string();
				name_par.push_str("(");
				name_par.into()
			},
		}
	}
}

pub enum Error {
	Nothing, // End of string, or nothing found
	BadChar(char),
	BadNum(big_dec::Error),
	FuncTooLong,
	FuncMissingPar
}

impl Error {
	pub fn to_string(&self) -> String {
		match *self {
			Error::Nothing => "".into(),
			Error::BadChar(c) => format!("bad char: '{}'", c),
			Error::BadNum(ref nverr) => nverr.to_string(),
			Error::FuncTooLong => "function name too long".into(),
			Error::FuncMissingPar => "missing function parenthesis".into(),
		}
	}
}

// Parses a function, made of a name (of type Name) followed immediately by an open parenthesis
fn parse_func(input_chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Name, Error> {
	let mut name = Name { len: 0, bytes: [0;MAX_NAME_LEN as usize] };
	let mut wants_first_letter = true;
	loop {
		let c = {
			let c_opt = input_chars.peek();
			if c_opt.is_none() {
				return Err(Error::Nothing); // End of string
			}
			let c = c_opt.unwrap();
			*c
		};

		if wants_first_letter {
			if 'a' <= c && c <= 'z' {
				// This is a function name
				wants_first_letter = false;
				name.bytes[name.len as usize] = c as u8; // We only handle ascii so casting to u8 is ok
				name.len += 1;
			}
			else {
				// Not a function, this is not an error
				return Err(Error::Nothing);
			}
		}
		else {
			if 'a' <= c && c <= 'z' {
				if name.len == MAX_NAME_LEN {
					// Name too long
					return Err(Error::FuncTooLong);
				}
				name.bytes[name.len as usize] = c as u8; // We only handle ascii so casting to u8 is ok
				name.len += 1;
			}
			else if c == '(' {
				// Done. Consume current char before leaving.
				input_chars.next();
				break;
			}
			else {
			    return Err(Error::FuncMissingPar);
			}

		}
		input_chars.next();
	}
	Ok(name)
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
			// Try to parse a number
			let num_res = BigDec::parse_chars(&mut self.input_chars);
			match num_res {
				Ok(num) => {
					ret = Ok(Token::Number(num));
					break;
				},
				Err(err) => {
					match err {
						big_dec::Error::ParseNothing => (), // Not a problem
						_ => { return Err(Error::BadNum(err)); }
					}
				}
			}

			// Try to parse a function
			let func_name_res = parse_func(&mut self.input_chars);
			match func_name_res {
				Ok(func_name) => {
					ret = Ok(Token::Func(func_name));
					break;
				},
				Err(err) => {
					match err {
						Error::Nothing => (), // Not a problem
						_ => { return Err(err); }
					}
				}
			}

			// Try other symbols
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
