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
	ParseBadChar,

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
			Error::ParseBadChar => "bad character".to_string(),
			Error::OpDivideByZero => "divide by zero".to_string(),
			Error::OpOverflow => "overflow".to_string()
		}
	}
}

#[derive(Copy)]
pub struct BigDec {
	neg: bool,
	digits: [u8;MAX_LEN], // little-endian. 1402.658 -> 0,0,0,...,8,5,6, 2,0,4,1,0,0,...,0
}

struct DivRet {
	quotient: BigDec,
	remainder: BigDec,
}

impl std::fmt::Display for BigDec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for BigDec {
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

impl Clone for BigDec {
	fn clone(&self) -> BigDec {
		*self
	}
}

impl BigDec {
	pub fn zero() -> BigDec {
		BigDec {
			neg: false,
			digits: [0;MAX_LEN]
		}
	}

	// for testing
	#[allow(dead_code)]
	pub fn from_i32(val: i32) -> BigDec {
		let mut ret = BigDec::zero();
		let mut val_u = if val < 0 {
			ret.neg = true;
			-val as u32
		}
		else {
			val as u32
		};

		if val_u < 10 {
			// Fast path
			ret.digits[FRAC_LEN] = val_u as u8;
		}
		else {
			// Slow path
			let mut index = FRAC_LEN;
			loop {
				if val_u == 0 {
					break;
				}
				let x = (val_u % 10) as u8;
				val_u /= 10;

				ret.digits[index] = x;
				index += 1;
			}
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

	pub fn max_len() -> usize {
		MAX_LEN
	}

	pub fn is_zero(&self) -> bool {
		for i in 0..MAX_LEN {
			if self.digits[i] != 0 {
				return false;
			}
		}
		true
	}

	pub fn negate(&self) -> BigDec {
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
	fn add_u(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		let mut nv1_digits = nv1.digits;
		try!(BigDec::accumulate_u(&nv0.digits, &mut nv1_digits));
		Ok(BigDec { neg: false, digits: nv1_digits })
	}

	// subtract without looking at the negative state of the inputs
	// the output can become negative
	fn sub_u(nv0: BigDec, nv1: BigDec) -> BigDec {
		let mut ret = BigDec::zero();
		let swap = BigDec::compare(nv0, nv1) == -1;
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

	pub fn add(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		if !nv0.neg && !nv1.neg {
			return BigDec::add_u(nv0, nv1);
		}
		else if nv0.neg && nv1.neg {
			let mut ret = match BigDec::add_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			ret.neg = true;
			return Ok(ret);
		}
		else if nv0.neg && !nv1.neg {
			return Ok(BigDec::sub_u(nv1, nv0));
		}
		// !nv0.neg && nv1.neg
		return Ok(BigDec::sub_u(nv0, nv1));
	}

	pub fn compare(nv0: BigDec, nv1: BigDec) -> i32 {
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

	pub fn sub(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		if nv0.neg && !nv1.neg {
			let mut ret = match BigDec::add_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			ret.neg = true;
			return Ok(ret);
		}
		else if nv0.neg && nv1.neg {
			return Ok(BigDec::sub_u(nv1, nv0));
		}
		else if !nv0.neg && nv1.neg {
			let ret = match BigDec::add_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			return Ok(ret);
		}
		// !nv0.neg && !nv1.neg
		return Ok(BigDec::sub_u(nv0, nv1));
	}

	// Multiplies one BigDec with one single digit
	// Returns the result as a list of digits that can be shifted
	fn mul_u_digit(nv0: &BigDec, digit: u8, shift: usize) -> [u8;MAX_LEN_MUL] {
		let mut line = [0u8;MAX_LEN_MUL];
		let mut carry = 0;
		for i in 0..MAX_LEN {
			let z = digit * nv0.digits[i] + carry;
			carry = z / 10;
			line[shift + i] = z % 10;
		}
		line
	}

	fn mul_u(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		let mut result = [0u8;MAX_LEN_MUL];
		for i in 0..MAX_LEN {
			let line = BigDec::mul_u_digit(&nv0, nv1.digits[i], i);
			try!(BigDec::accumulate_u(&line, &mut result)); // Cannot overflow actually
		}
		let mut ret = BigDec::zero();
		// Check overflow
		for i in FRAC_LEN+MAX_LEN..MAX_LEN_MUL {
			if result[i] != 0 {
				return Err(Error::OpOverflow);
			}
		}
		ret.digits.copy_from_slice(&result[FRAC_LEN..FRAC_LEN+MAX_LEN]);
		Ok(ret)
	}

	pub fn mul(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		if nv0.neg == nv1.neg {
			return BigDec::mul_u(nv0, nv1);
		}
		else {
			let mut ret = match BigDec::mul_u(nv0, nv1) {
				Ok(nv) => nv,
				Err(err) => { return Err(err); }
			};
			ret.neg = true;
			return Ok(ret);
		}
	}

	// Multiply by 10
	fn shift_right(&mut self) {
		for i in 0..MAX_LEN-1 {
			self.digits[MAX_LEN-1-i] = self.digits[MAX_LEN-2-i];
		}
		self.digits[0] = 0;
	}

	// Divide by 10
	#[allow(dead_code)]
	fn shift_left(&mut self) {
		for i in 0..MAX_LEN-1 {
			self.digits[i] = self.digits[i + 1];
		}
		self.digits[MAX_LEN - 1] = 0;
	}

	fn div_u(nv0: BigDec, nv1: BigDec, with_frac: bool) -> DivRet {
		//println!("u_div: {:?} / {:?}", nv0, nv1);
		let mut tmp = BigDec::zero();
		let mut src_digit_index = Some(MAX_LEN - 1);
		let mut index = 0;
		let stop_at = if with_frac { MAX_LEN+FRAC_LEN } else { MAX_LEN };

		let mut result = BigDec::zero();

		loop {
			src_digit_index = match src_digit_index {
				Some(index) => {
					tmp.digits[0] = nv0.digits[index];
					if index == 0 {
						None
					}
					else {
						Some(index - 1)
					}
				},
				None => None
			};

			let mut counter = 0;
			//println!("div_u: enter subloop");
			loop {
				let next_diff = BigDec::sub_u(tmp, nv1);
				//println!("div_u: {} - {} = {}", tmp, nv1, next_diff);
				if next_diff.neg {
					//println!("div_u: subloop stop, counter: {}", counter);
					break;
				}
				tmp = next_diff;
				counter += 1;
			}
			result.shift_right();
			result.digits[0] = counter;

			index += 1;
			if index == stop_at {
				break;
			}

			tmp.neg = false;
			tmp.shift_right();

		}
		if with_frac {
			tmp.shift_right();
		}
		DivRet { quotient: result, remainder: tmp }
	}

	pub fn div(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		if nv1.is_zero() {
			return Err(Error::OpDivideByZero);
		}

		if nv0.neg {
			if nv1.neg {
				let div_ret = BigDec::div_u(nv0, nv1, true);
				return Ok(div_ret.quotient);
			}
			else {
				let mut div_ret = BigDec::div_u(nv0, nv1, true);
				div_ret.quotient.neg = true;
				return Ok(div_ret.quotient);
			}
		}
		else {
			if nv1.neg {
				let mut div_ret = BigDec::div_u(nv0, nv1, true);
				div_ret.quotient.neg = true;
				return Ok(div_ret.quotient);
			}
			else {
				let div_ret = BigDec::div_u(nv0, nv1, true);
				return Ok(div_ret.quotient);
			}
		}
	}

	pub fn div_mod(nv0: BigDec, nv1: BigDec) -> Result<BigDec, Error> {
		if nv1.is_zero() {
			return Err(Error::OpDivideByZero);
		}

		if nv1.neg {
			let mut div_ret = BigDec::div_u(nv0, nv1, false);
			div_ret.remainder.neg = true;
			return Ok(div_ret.remainder);
		}
		else {
			let div_ret = BigDec::div_u(nv0, nv1, false);
			return Ok(div_ret.remainder);
		}
	}

	pub fn fact(mut n: BigDec) -> Result<BigDec, Error> {
		if n.is_zero() {
			return Ok(BigDec::from_i32(1));
		}
		// Clear fractional part
		for i in 0..FRAC_LEN {
			n.digits[i] = 0;
		}
		// Clear sign
		let sign = n.neg;
		n.neg = false;
		let one = BigDec::from_i32(1);
		let mut val = n;
		loop {
			let n_minus_one = BigDec::sub_u(n, one);
			if n_minus_one.is_zero() {
				break;
			}
			let mul_res = BigDec::mul_u(val, n_minus_one);
			if mul_res.is_err() {
				return mul_res;
			}
			val = mul_res.unwrap();
			n = n_minus_one;
		}
		val.neg = sign;
		Ok(val)
	}

	// Converts a BigDec which internal representation uses a base 16
	// to a regular BigDec which uses a base 10
	fn hex_to_dec(&self) -> Result<BigDec, Error> {
		// Get the number of hex digits
		let mut limit = MAX_LEN;
		for i in (FRAC_LEN..MAX_LEN).rev() {
			if self.digits[i] != 0 {
				break;
			}
			limit -= 1;
		}
		//println!("limit: {}", limit);

		let mut comp_result = BigDec::zero();
		let sixteen = BigDec::from_i32(16);
		let mut power_of_16 = BigDec::from_i32(1);

		for i in FRAC_LEN..limit {
			let digit = self.digits[i] as i32;
			let bd_digit = BigDec::from_i32(digit);
			let mul = try!(BigDec::mul(bd_digit, power_of_16));
			comp_result = try!(BigDec::add(comp_result, mul));
			//println!("{}: {}", i - FRAC_LEN, comp_result);
			power_of_16 = try!(BigDec::mul(power_of_16, sixteen));
			//println!("power of 16: {}", power_of_16);
		}
		Ok(comp_result)
	}

	// Parses a positive number
	// The digits can be separated with an underscore
	// ex: 14_950.234_845
	pub fn parse_chars(input_chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<BigDec, Error> {
		let c = {
			let c_opt = input_chars.peek();
			if c_opt.is_none() {
				return Err(Error::ParseNothing); // End of string
			}
			let c = c_opt.unwrap();
			*c
		};

		// Check that the character is a digit
		let first_digit32 = {
			if !c.is_digit(10) {
				return Result::Err(Error::ParseNothing); // Not a number
			}
			c.to_digit(10).unwrap() as u8
		};
		input_chars.next();

		let mut val = BigDec::zero();
		val.digits[FRAC_LEN] = first_digit32;

		let mut shift_count = 1;
		let mut dot_found = false;
		let mut sep_found = false;
		let mut radix = 10;
		let mut radix_found = false;
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

			if c == 'x' {
				if !radix_found && shift_count == 1 && first_digit32 == 0 {
					// Hexadecimal
					radix = 16;
					radix_found = true;
				}
				else {
				    return Err(Error::ParseBadChar);
				}
			}
			else if c == '.' {
				if dot_found {
					// Already found, exit
					break;
				}
				dot_found = true;
			}
			else if c == '_' {
				if sep_found {
					// Double '_' found
					return Err(Error::ParseBadChar);
				}
				sep_found = true;
			}
			else {
				let digit32 = {
					if !c.is_digit(radix) {
						break;
					}
					c.to_digit(radix).unwrap() as u8
				};
				// Reset separator status
				sep_found = false;

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
		println!("parse ok");
		if radix == 16 {
			// Convert the collected values
			val.hex_to_dec()
		}
		else {
			Result::Ok(val)
		}
	}

	// for testing
	#[allow(dead_code)]
	pub fn parse_str(arg: &str) -> Result<BigDec, Error> {
		let mut ic = arg.chars().peekable();
		BigDec::parse_chars(&mut ic)
	}
}

impl PartialEq for BigDec {
    fn eq(&self, other: &BigDec) -> bool {
        BigDec::compare(*self, *other) == 0
    }
}

#[test]
fn test_add() {
	assert_eq!("3",  BigDec::add(BigDec::from_i32(1),  BigDec::from_i32(2)).unwrap().to_string());
	assert_eq!("-3", BigDec::add(BigDec::from_i32(-1), BigDec::from_i32(-2)).unwrap().to_string());
	assert_eq!("1",  BigDec::add(BigDec::from_i32(-1), BigDec::from_i32(2)).unwrap().to_string());
	assert_eq!("-1", BigDec::add(BigDec::from_i32(1),  BigDec::from_i32(-2)).unwrap().to_string());

	let mut less_than_zero = BigDec::zero();
	less_than_zero.digits[FRAC_LEN-1] = 1;
	assert_eq!("0.1", less_than_zero.to_string());
}

#[test]
fn test_add_overflow() {
	let mut arg = String::new();
	for _ in 0..INT_LEN {
		arg.push('9');
	}
	let res = BigDec::add(BigDec::parse_str(&arg).unwrap(),  BigDec::parse_str(&arg).unwrap());
	assert!(res.is_err());
	let expected_err = match res.unwrap_err() {
		Error::OpOverflow => true,
		_ => false
	};
	assert!(expected_err);
}

#[test]
fn test_sub() {
	assert_eq!("1",  BigDec::sub(BigDec::from_i32(2),  BigDec::from_i32(1)).unwrap().to_string());
	assert_eq!("-1", BigDec::sub(BigDec::from_i32(-2), BigDec::from_i32(-1)).unwrap().to_string());
	assert_eq!("-3", BigDec::sub(BigDec::from_i32(-2), BigDec::from_i32(1)).unwrap().to_string());
	assert_eq!("3",  BigDec::sub(BigDec::from_i32(2),  BigDec::from_i32(-1)).unwrap().to_string());

	assert_eq!("-1", BigDec::sub(BigDec::from_i32(1),  BigDec::from_i32(2)).unwrap().to_string());
	assert_eq!("1",  BigDec::sub(BigDec::from_i32(-1), BigDec::from_i32(-2)).unwrap().to_string());
	assert_eq!("-3", BigDec::sub(BigDec::from_i32(-1), BigDec::from_i32(2)).unwrap().to_string());
	assert_eq!("3",  BigDec::sub(BigDec::from_i32(1),  BigDec::from_i32(-2)).unwrap().to_string());
}

#[test]
fn test_mul() {
	assert_eq!("15",  BigDec::mul(BigDec::from_i32(3),  BigDec::from_i32(5)).unwrap().to_string());
	assert_eq!("150",  BigDec::mul(BigDec::from_i32(30),  BigDec::from_i32(5)).unwrap().to_string());
	assert_eq!("1500",  BigDec::mul(BigDec::from_i32(30),  BigDec::from_i32(50)).unwrap().to_string());
	assert_eq!("9801",  BigDec::mul(BigDec::from_i32(99),  BigDec::from_i32(99)).unwrap().to_string());

	assert_eq!("28",  BigDec::mul(BigDec::from_i32(4),  BigDec::from_i32(7)).unwrap().to_string());
	assert_eq!("-28",  BigDec::mul(BigDec::from_i32(-4),  BigDec::from_i32(7)).unwrap().to_string());
	assert_eq!("-28",  BigDec::mul(BigDec::from_i32(4),  BigDec::from_i32(-7)).unwrap().to_string());
	assert_eq!("28",  BigDec::mul(BigDec::from_i32(-4),  BigDec::from_i32(-7)).unwrap().to_string());
}

#[test]
fn test_mul_overflow() {
	let mut arg = String::new();
	for _ in 0..INT_LEN {
		arg.push('9');
	}
	let res = BigDec::mul(BigDec::parse_str(&arg).unwrap(),  BigDec::parse_str(&arg).unwrap());
	assert!(res.is_err());
	let expected_err = match res.unwrap_err() {
		Error::OpOverflow => true,
		_ => false
	};
	assert!(expected_err);
}

#[test]
fn test_div() {
	assert_eq!("15.625",  BigDec::div(BigDec::from_i32(1000),  BigDec::from_i32(64)).unwrap().to_string());
	assert_eq!("-15.625",  BigDec::div(BigDec::from_i32(-1000),  BigDec::from_i32(64)).unwrap().to_string());
	assert_eq!("-15.625",  BigDec::div(BigDec::from_i32(1000),  BigDec::from_i32(-64)).unwrap().to_string());
	assert_eq!("15.625",  BigDec::div(BigDec::from_i32(-1000),  BigDec::from_i32(-64)).unwrap().to_string());

	let mut one_point_six = BigDec::from_i32(16);
	one_point_six.shift_left();
	assert_eq!("1.25", BigDec::div(BigDec::from_i32(2), one_point_six).unwrap().to_string());
}

#[test]
fn test_div_mod() {
	assert_eq!("4",  BigDec::div_mod(BigDec::from_i32(100),  BigDec::from_i32(48)).unwrap().to_string());
	assert_eq!("4",  BigDec::div_mod(BigDec::from_i32(-100),  BigDec::from_i32(48)).unwrap().to_string());
	assert_eq!("-4",  BigDec::div_mod(BigDec::from_i32(100),  BigDec::from_i32(-48)).unwrap().to_string());
	assert_eq!("-4",  BigDec::div_mod(BigDec::from_i32(-100),  BigDec::from_i32(-48)).unwrap().to_string());
}

#[test]
fn test_fact() {
	assert_eq!("120",  BigDec::fact(BigDec::from_i32(5)).unwrap().to_string());
	assert_eq!("-120",  BigDec::fact(BigDec::from_i32(-5)).unwrap().to_string());
}

#[test]
fn test_parse_very_big() {
	let nv = BigDec::parse_str("1.02");
	assert!(nv.is_ok());
	assert_eq!("1.02", nv.unwrap().to_string());

	// Too many digits
	let mut too_long = String::with_capacity(INT_LEN + 1);
	for _ in 0..INT_LEN - 1 {
		too_long.push_str("1");
	}
	too_long.push_str("2");
	let nv = BigDec::parse_str(&too_long);
	assert!(nv.is_ok());

	too_long.push_str("3");
	let nv2 = BigDec::parse_str(&too_long);
	let is_int_part_overflow_err = match nv2 {
		Err(err) => match err {
			Error::ParseIntPartOverflow => true,
			_ => false
		},
		Ok(_) => false
	};
	assert!(is_int_part_overflow_err);
}

#[test]
fn test_parse_underscore() {
	let nv = BigDec::parse_str("1_000");
	assert!(nv.is_ok());
	assert_eq!("1000", nv.unwrap().to_string());

	let nv2 = BigDec::parse_str("1_234_567");
	assert!(nv2.is_ok());
	assert_eq!("1234567", nv2.unwrap().to_string());
}

#[test]
fn test_parse_hex() {
	let nv = BigDec::parse_str("0xffff");
	assert!(nv.is_ok());
	assert_eq!("65535", nv.unwrap().to_string());
}
