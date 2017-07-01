use std;
use std::char;

const INT_LEN: usize = 20;  // number of digits used for the integer part
const FRAC_LEN: usize = 20; // number of digits used for the fractional part

const MAX_LEN: usize = FRAC_LEN + INT_LEN;
const MAX_LEN_MUL: usize = MAX_LEN*2+1; // Max len for multiplication result

// for debugging
#[allow(dead_code)]
fn print_digits(digits: &[u8]) {
	print!("{}", digits[0]);
	for i in 1..digits.len() {
		print!(",{}", digits[i]);
	}
	println!("");
}

fn find_bounds(digits: &[u8]) -> (usize, usize) {
	let mut start_at = 0;
	while start_at < FRAC_LEN {
		if digits[start_at] != 0 {
			break;
		}
		start_at += 1;
	}

	let mut stop_at = digits.len();
	while stop_at > FRAC_LEN + 1 {
		if digits[stop_at-1] != 0 {
			break;
		}
		stop_at -= 1;
	}
	(start_at, stop_at)
}

#[derive(Debug)]
pub enum Error {
	// parsing
	ParseNothing,
	ParseIntPartOverflow,
	ParseFracPartOverflow,

	// operations
	OpDivideByZero,
	OpOverflow
}

impl Error {
	pub fn to_string(&self) -> String {
		match *self {
			Error::ParseNothing => "".to_string(),
			Error::ParseIntPartOverflow => "too many digits".to_string(),
			Error::ParseFracPartOverflow => "too many decimals".to_string(),
			Error::OpDivideByZero => "divide by zero".to_string(),
			Error::OpOverflow => "overflow".to_string()
		}
	}
}

#[derive(Copy)]
pub struct NumVal {
	neg: bool,
	digits: [u8;MAX_LEN], // little-endian. 1402.658 -> 0,0,0,...,8,5,6, 2,0,4,1,0,0,...,0
}

struct DivRet {
	quotient: NumVal,
	remainder: NumVal,
}

impl std::fmt::Display for NumVal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for NumVal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut ds = String::with_capacity(MAX_LEN+3);
        for i in 0..FRAC_LEN {
			let x = self.digits[i] as u32;
			let c = char::from_u32(48 + x).unwrap();
			ds.push(c);
			if i < (FRAC_LEN - 1) {
				ds.push(',');
			}
        }
        ds.push_str(" . ");
        for i in FRAC_LEN..MAX_LEN {
			let x = self.digits[i] as u32;
			let c = char::from_u32(48 + x).unwrap();
			ds.push(c);
			if i < (MAX_LEN - 1) {
				ds.push(',');
			}
        }
        write!(f, "neg: {} digits: {}", self.neg, ds)
	}
}

impl Clone for NumVal {
	fn clone(&self) -> NumVal {
		*self
	}
}

impl NumVal {
	pub fn zero() -> NumVal {
		NumVal {
			neg: false,
			digits: [0;MAX_LEN]
		}
	}

	// for testing
	#[allow(dead_code)]
	pub fn from_i32(val: i32) -> NumVal {
		let mut ret = NumVal::zero();
		let mut index = FRAC_LEN;
		let mut val_u = if val < 0 {
			ret.neg = true;
			-val as u32
		}
		else {
			ret.neg = false;
			val as u32 
		};

		loop {
			if val_u == 0 {
				break;
			}
			let x = (val_u % 10) as u8;
			val_u /= 10;

			ret.digits[index] = x;
			index += 1;
		}
		ret
	}

	pub fn to_string(&self) -> String {
		let mut ret = String::with_capacity(MAX_LEN + 2);
		let (start_at, stop_at) = find_bounds(&self.digits);

		if start_at == stop_at {
			ret.push('0');
		}
		else {
			if self.neg {
				ret.push('-');
			}

			for i in (FRAC_LEN..stop_at).rev() {
				let x = self.digits[i] as u32;
				let c = char::from_u32(48 + x).unwrap();
				ret.push(c);
			}

			if start_at < FRAC_LEN {
				ret.push('.');
				for i in (start_at..FRAC_LEN).rev() {
					let x = self.digits[i] as u32;
					let c = char::from_u32(48 + x).unwrap();
					ret.push(c);
				}
			}
		}
		ret
	}

	pub fn is_zero(&self) -> bool {
		for i in 0..MAX_LEN {
			if self.digits[i] != 0 {
				return false;
			}
		}
		true
	}

	pub fn negate(&self) -> NumVal {
		let mut ret = *self;
		ret.neg = !ret.neg;
		ret
	}

	// add arbitrary length but consistent among all arguments
	// digits0: digits to add to digits1
	// digits1: existing values
	fn accumulate_u(digits0: &[u8], digits1: &mut [u8]) -> Result<(), Error> {
		let mut carry = 0;
		for i in 0..digits0.len() {
			let z = digits0[i] + digits1[i] + carry;
			carry = z / 10;
			let z2 = z % 10;
			digits1[i] = z2;
		}
		if carry != 0 {
			Err(Error::OpOverflow)
		}
		else {
			Ok(())
		}
	}

	// add without looking at the negative state of the inputs
	fn add_u(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		let mut nv1_digits = nv1.digits;
		try!(NumVal::accumulate_u(&nv0.digits, &mut nv1_digits));
		Ok(NumVal { neg: false, digits: nv1_digits })
	}

	// subtract without looking at the negative state of the inputs
	// the output can become negative
	fn sub_u(nv0: NumVal, nv1: NumVal) -> NumVal {
		let mut ret = NumVal::zero();
		let swap = NumVal::compare(nv0, nv1) == -1;
		let (nv_left, nv_right) = if swap { (nv1, nv0) } else { (nv0, nv1) }; 
		let mut carry = 0;
		for i in 0..MAX_LEN {
			let x = nv_left.digits[i];
			let y = nv_right.digits[i];
			let z;
			if x >= (y + carry) {
				z = x - (y + carry);
				carry = 0;
			}
			else {
				z = (10 + x) - (y + carry);
				carry = 1;
			}
			ret.digits[i] = z;
		}
		ret.neg = swap;
		ret
	}

	pub fn add(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		if !nv0.neg && !nv1.neg {
			return NumVal::add_u(nv0, nv1);
		}
		else if nv0.neg && nv1.neg {
			let mut ret = match NumVal::add_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			ret.neg = true;
			return Ok(ret);
		}
		else if nv0.neg && !nv1.neg {
			return Ok(NumVal::sub_u(nv1, nv0));
		}
		// !nv0.neg && nv1.neg
		return Ok(NumVal::sub_u(nv0, nv1));
	}

	fn compare(nv0: NumVal, nv1: NumVal) -> i32 {
		for i in (0..MAX_LEN).rev() {
			let x = nv0.digits[i];
			let y = nv1.digits[i];
			if x < y {
				return -1;
			}
			else if x > y {
				return 1;
			}
		}
		return 0;
	}

	pub fn sub(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		if nv0.neg && !nv1.neg {
			let mut ret = match NumVal::add_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			ret.neg = true;
			return Ok(ret);
		}
		else if nv0.neg && nv1.neg {
			return Ok(NumVal::sub_u(nv1, nv0));
		}
		else if !nv0.neg && nv1.neg {
			let ret = match NumVal::add_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			return Ok(ret);
		}
		// !nv0.neg && !nv1.neg
		return Ok(NumVal::sub_u(nv0, nv1));
	}

	// Multiplies one NumVal with one single digit
	// Returns the result as a list of digits that can be shifted 
	fn mul_u_digit(nv0: &NumVal, digit: u8, shift: usize) -> [u8;MAX_LEN_MUL] {
		let mut line = [0u8;MAX_LEN_MUL];
		let mut carry = 0;
		for i in 0..MAX_LEN {
			let z = digit * nv0.digits[i] + carry;
			carry = z / 10;
			line[shift + i] = z % 10;
		}
		line
	}

	fn mul_u(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		let mut result = [0u8;MAX_LEN_MUL];
		for i in 0..MAX_LEN {
			let line = NumVal::mul_u_digit(&nv0, nv1.digits[i], i);
			try!(NumVal::accumulate_u(&line, &mut result)); // Cannot overflow actually
		}
		let mut ret = NumVal::zero();
		// Check overflow
		for i in FRAC_LEN+MAX_LEN..MAX_LEN_MUL {
			if result[i] != 0 {
				return Err(Error::OpOverflow);
			}
		}
		ret.digits.copy_from_slice(&result[FRAC_LEN..FRAC_LEN+MAX_LEN]);
		Ok(ret)
	}

	pub fn mul(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		if nv0.neg == nv1.neg {
			return NumVal::mul_u(nv0, nv1);
		}
		else {
			let mut ret = match NumVal::mul_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			ret.neg = true;
			return Ok(ret);
		}
	}

	fn shift_right(&mut self) {
		for i in 0..MAX_LEN-1 {
			self.digits[MAX_LEN-1-i] = self.digits[MAX_LEN-2-i];
		}
		self.digits[0] = 0;
	}

	fn div_u(nv0: NumVal, nv1: NumVal, with_frac: bool) -> DivRet {
		let mut tmp = NumVal::zero();
		let mut i = MAX_LEN - 1;
		let stop_at = if with_frac { 0 } else { FRAC_LEN };
		let mut result = NumVal::zero();

		loop {
			tmp.digits[FRAC_LEN] = nv0.digits[i];

			let mut counter = 0;
			loop {
				let next_diff = NumVal::sub_u(tmp, nv1);
				if next_diff.neg {
					break;
				}
				tmp = next_diff;
				counter += 1;
			}
			result.shift_right();
			result.digits[0] = counter;
			
			if i == stop_at {
				break;
			}
			i -= 1;
			tmp.neg = false;
			tmp.shift_right();
		}
		DivRet { quotient: result, remainder: tmp }
	}

	pub fn div(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		if nv1.is_zero() {
			return Err(Error::OpDivideByZero);
		}

		if nv0.neg {
			if nv1.neg {
				let div_ret = NumVal::div_u(nv0, nv1, true);
				return Ok(div_ret.quotient);
			}
			else {
				let mut div_ret = NumVal::div_u(nv0, nv1, true);
				div_ret.quotient.neg = true;
				return Ok(div_ret.quotient);
			}
		}
		else {
			if nv1.neg {
				let mut div_ret = NumVal::div_u(nv0, nv1, true);
				div_ret.quotient.neg = true;
				return Ok(div_ret.quotient);
			}
			else {
				let div_ret = NumVal::div_u(nv0, nv1, true);
				return Ok(div_ret.quotient);
			}
		}
	}

	pub fn div_mod(nv0: NumVal, nv1: NumVal) -> Result<NumVal, Error> {
		if nv1.is_zero() {
			return Err(Error::OpDivideByZero);
		}

		if nv1.neg {
			let mut div_ret = NumVal::div_u(nv0, nv1, false);
			div_ret.remainder.neg = true;
			return Ok(div_ret.remainder);
		}
		else {
			let div_ret = NumVal::div_u(nv0, nv1, false);
			return Ok(div_ret.remainder);
		}
	}

	// Parses a positive number
	pub fn parse_chars(input_chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<NumVal, Error> {
		let c = {
			let c_opt = input_chars.peek();
			if c_opt.is_none() {
				return Err(Error::ParseNothing); // No character
			}
			let c = c_opt.unwrap();
			*c
		};

		// Check that the character is a digit
		let digit32 = {
			if !c.is_digit(10) {
				return Result::Err(Error::ParseNothing);
			}
			c.to_digit(10).unwrap() as u8
		};
		input_chars.next();
		
		let mut val = NumVal::zero();
		val.digits[FRAC_LEN] = digit32;

		let mut shift_count = 0;
		let mut dot_found = false;
		let mut frac_index = FRAC_LEN;
		loop {
			let c = {
				let c_opt = input_chars.peek();
				if c_opt.is_none() {
					break;
				}
				let c = c_opt.unwrap();
				*c
			};

			if c == '.' {
				if dot_found {
					// Already found, exit
					break;
				}
				dot_found = true;
			}
			else {
				let digit32 = {
					if !c.is_digit(10) {
						break;
					}
					c.to_digit(10).unwrap() as u8
				};

				if !dot_found {
					if shift_count == INT_LEN {
						return Err(Error::ParseIntPartOverflow);
					}
					val.shift_right();
					val.digits[FRAC_LEN] = digit32;
					shift_count += 1;
				}
				else {
					if frac_index == 0 {
						return Err(Error::ParseFracPartOverflow);
					}
					frac_index -= 1;
					val.digits[frac_index] = digit32;
				}
			}
			input_chars.next();
		}
		Result::Ok(val)
	}

	// for testing
	#[allow(dead_code)]
	pub fn parse_str(arg: &str) -> Result<NumVal, Error> {
		let mut ic = arg.chars().peekable();
		NumVal::parse_chars(&mut ic)
	}
}

#[test]
fn test_add() {
	assert_eq!("3",  NumVal::add(NumVal::from_i32(1),  NumVal::from_i32(2)).unwrap().to_string());
	assert_eq!("-3", NumVal::add(NumVal::from_i32(-1), NumVal::from_i32(-2)).unwrap().to_string());
	assert_eq!("1",  NumVal::add(NumVal::from_i32(-1), NumVal::from_i32(2)).unwrap().to_string());
	assert_eq!("-1", NumVal::add(NumVal::from_i32(1),  NumVal::from_i32(-2)).unwrap().to_string());

	let mut less_than_zero = NumVal::zero();
	less_than_zero.digits[FRAC_LEN-1] = 1;
	assert_eq!("0.1", less_than_zero.to_string());
}

#[test]
fn test_add_overflow() {
	let mut arg = String::new();
	for _ in 0..INT_LEN {
		arg.push('9');
	}
	let res = NumVal::add(NumVal::parse_str(&arg).unwrap(),  NumVal::parse_str(&arg).unwrap());
	assert!(res.is_err());
	let expected_err = match res.unwrap_err() {
		Error::OpOverflow => true,
		_ => false
	};
	assert!(expected_err);
}

#[test]
fn test_sub() {
	assert_eq!("1",  NumVal::sub(NumVal::from_i32(2),  NumVal::from_i32(1)).unwrap().to_string());
	assert_eq!("-1", NumVal::sub(NumVal::from_i32(-2), NumVal::from_i32(-1)).unwrap().to_string());
	assert_eq!("-3", NumVal::sub(NumVal::from_i32(-2), NumVal::from_i32(1)).unwrap().to_string());
	assert_eq!("3",  NumVal::sub(NumVal::from_i32(2),  NumVal::from_i32(-1)).unwrap().to_string());

	assert_eq!("-1", NumVal::sub(NumVal::from_i32(1),  NumVal::from_i32(2)).unwrap().to_string());
	assert_eq!("1",  NumVal::sub(NumVal::from_i32(-1), NumVal::from_i32(-2)).unwrap().to_string());
	assert_eq!("-3", NumVal::sub(NumVal::from_i32(-1), NumVal::from_i32(2)).unwrap().to_string());
	assert_eq!("3",  NumVal::sub(NumVal::from_i32(1),  NumVal::from_i32(-2)).unwrap().to_string());
}

#[test]
fn test_mul() {
	assert_eq!("15",  NumVal::mul(NumVal::from_i32(3),  NumVal::from_i32(5)).unwrap().to_string());
	assert_eq!("150",  NumVal::mul(NumVal::from_i32(30),  NumVal::from_i32(5)).unwrap().to_string());
	assert_eq!("1500",  NumVal::mul(NumVal::from_i32(30),  NumVal::from_i32(50)).unwrap().to_string());
	assert_eq!("9801",  NumVal::mul(NumVal::from_i32(99),  NumVal::from_i32(99)).unwrap().to_string());

	assert_eq!("28",  NumVal::mul(NumVal::from_i32(4),  NumVal::from_i32(7)).unwrap().to_string());
	assert_eq!("-28",  NumVal::mul(NumVal::from_i32(-4),  NumVal::from_i32(7)).unwrap().to_string());
	assert_eq!("-28",  NumVal::mul(NumVal::from_i32(4),  NumVal::from_i32(-7)).unwrap().to_string());
	assert_eq!("28",  NumVal::mul(NumVal::from_i32(-4),  NumVal::from_i32(-7)).unwrap().to_string());
}

#[test]
fn test_mul_overflow() {
	let mut arg = String::new();
	for _ in 0..INT_LEN {
		arg.push('9');
	}
	let res = NumVal::mul(NumVal::parse_str(&arg).unwrap(),  NumVal::parse_str(&arg).unwrap());
	assert!(res.is_err());
	let expected_err = match res.unwrap_err() {
		Error::OpOverflow => true,
		_ => false
	};
	assert!(expected_err);
}

#[test]
fn test_div() {
	assert_eq!("15.625",  NumVal::div(NumVal::from_i32(1000),  NumVal::from_i32(64)).unwrap().to_string());
	assert_eq!("-15.625",  NumVal::div(NumVal::from_i32(-1000),  NumVal::from_i32(64)).unwrap().to_string());
	assert_eq!("-15.625",  NumVal::div(NumVal::from_i32(1000),  NumVal::from_i32(-64)).unwrap().to_string());
	assert_eq!("15.625",  NumVal::div(NumVal::from_i32(-1000),  NumVal::from_i32(-64)).unwrap().to_string());
}

#[test]
fn test_div_mod() {
	assert_eq!("4",  NumVal::div_mod(NumVal::from_i32(100),  NumVal::from_i32(48)).unwrap().to_string());
	assert_eq!("4",  NumVal::div_mod(NumVal::from_i32(-100),  NumVal::from_i32(48)).unwrap().to_string());
	assert_eq!("-4",  NumVal::div_mod(NumVal::from_i32(100),  NumVal::from_i32(-48)).unwrap().to_string());
	assert_eq!("-4",  NumVal::div_mod(NumVal::from_i32(-100),  NumVal::from_i32(-48)).unwrap().to_string());
}

#[test]
fn test_parse() {
	let nv = NumVal::parse_str("1.02");
	assert!(nv.is_ok());
	assert_eq!("1.02", nv.unwrap().to_string());
}
