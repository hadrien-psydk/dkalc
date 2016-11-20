use std;
use std::char;

const INT_LEN: usize = 6;  // number of digits used for the integer part
const FRAC_LEN: usize = 3; // number of digits used for the fractional part

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
	while stop_at > start_at {
		if digits[stop_at-1] != 0 {
			break;
		}
		stop_at -= 1;
	}
	(start_at, stop_at)
}

#[derive(Copy,Clone,Debug)]
pub struct NumVal {
	neg: bool,
	digits: [u8;MAX_LEN], // little-endian. 1402.658 -> 0,0,0,...,8,5,6,2,0,4,1,..0
}

struct DivRet {
	quotient: NumVal,
	remainder: NumVal,
}

impl NumVal {

	pub fn zero() -> NumVal {
		NumVal {
			neg: false,
			digits: [0;MAX_LEN]
		}
	}

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
		//println!("bounds: {} {}", start_at, stop_at);

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
		let (start_at, stop_at) = find_bounds(&self.digits);
		start_at == stop_at
	}

	pub fn negate(&self) -> NumVal {
		let mut ret = *self;
		ret.neg = !ret.neg;
		ret
	}

	// add arbitrary length but consistent among all arguments
	// digits0: digits to add to digits1
	// digits1: existing values
	fn accumulate_u(digits0: &[u8], digits1: &mut [u8]) {
		let mut carry = 0;
		for i in 0..digits0.len() {
			let z = digits0[i] + digits1[i] + carry;
			carry = z / 10;
			let z2 = z % 10;
			digits1[i] = z2;
		}
	}

	// add without looking at the negative state of the inputs
	fn add_u(nv0: NumVal, nv1: NumVal) -> NumVal {
		let mut nv1_digits = nv1.digits;
		NumVal::accumulate_u(&nv0.digits, &mut nv1_digits);
		NumVal { neg: false, digits: nv1_digits }
	}

	// subtract without looking at the negative state of the inputs
	// the output can become negative
	fn sub_u(nv0: NumVal, nv1: NumVal) -> NumVal {
		let mut ret = NumVal::zero();
		let swap = NumVal::compare(nv0, nv1) == -1;
		let mut carry = 0;
		for i in 0..MAX_LEN {
			let x = nv0.digits[i];
			let y = nv1.digits[i];
			let (x, y) = if swap { (y, x) } else { (x, y) };

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

	pub fn add(nv0: NumVal, nv1: NumVal) -> NumVal {
		if !nv0.neg && !nv1.neg {
			return NumVal::add_u(nv0, nv1);
		}
		else if nv0.neg && nv1.neg {
			let mut ret = NumVal::add_u(nv0, nv1);
			ret.neg = true;
			return ret;
		}
		else if nv0.neg && !nv1.neg {
			return NumVal::sub_u(nv1, nv0);
		}
		// !nv0.neg && nv1.neg
		return NumVal::sub_u(nv0, nv1);
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

	pub fn sub(nv0: NumVal, nv1: NumVal) -> NumVal {
		if nv0.neg && !nv1.neg {
			let mut ret = NumVal::add_u(nv0, nv1);
			ret.neg = true;
			return ret;
		}
		else if nv0.neg && nv1.neg {
			return NumVal::sub_u(nv1, nv0);
		}
		else if !nv0.neg && nv1.neg {
			return NumVal::add_u(nv0, nv1);
		}
		// !nv0.neg && !nv1.neg
		return NumVal::sub_u(nv0, nv1);
	}

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

	fn mul_u(nv0: NumVal, nv1: NumVal) -> NumVal {
		let mut result = [0u8;MAX_LEN_MUL];
		for i in 0..MAX_LEN {
			let line = NumVal::mul_u_digit(&nv0, nv1.digits[i], i);
			NumVal::accumulate_u(&line, &mut result);
		}
		let mut ret = NumVal::zero();
		ret.digits.copy_from_slice(&result[FRAC_LEN..FRAC_LEN+MAX_LEN]);
		ret
	}

	pub fn mul(nv0: NumVal, nv1: NumVal) -> NumVal {
		if nv0.neg {
			if nv1.neg {
				return NumVal::mul_u(nv0, nv1);
			}
			else {
				let mut ret = NumVal::mul_u(nv0, nv1);
				ret.neg = true;
				return ret;
			}
		}
		else {
			if nv1.neg {
				let mut ret = NumVal::mul_u(nv0, nv1);
				ret.neg = true;
				return ret;
			}
			else {
				return NumVal::mul_u(nv0, nv1);
			}
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

	pub fn div(nv0: NumVal, nv1: NumVal) -> NumVal {
		if nv0.neg {
			if nv1.neg {
				let div_ret = NumVal::div_u(nv0, nv1, true);
				return div_ret.quotient;
			}
			else {
				let mut div_ret = NumVal::div_u(nv0, nv1, true);
				div_ret.quotient.neg = true;
				return div_ret.quotient;
			}
		}
		else {
			if nv1.neg {
				let mut div_ret = NumVal::div_u(nv0, nv1, true);
				div_ret.quotient.neg = true;
				return div_ret.quotient;
			}
			else {
				let div_ret = NumVal::div_u(nv0, nv1, true);
				return div_ret.quotient;
			}
		}
	}

	pub fn div_mod(nv0: NumVal, nv1: NumVal) -> NumVal {
		if nv1.neg {
			let mut div_ret = NumVal::div_u(nv0, nv1, false);
			div_ret.remainder.neg = true;
			return div_ret.remainder;
		}
		else {
			let div_ret = NumVal::div_u(nv0, nv1, false);
			return div_ret.remainder;
		}
	}

	pub fn parse(input_chars: &mut std::iter::Peekable<std::str::Chars>, c_first: char) -> Option<NumVal> {
		let mut val = 0;
		val += c_first.to_digit(10).unwrap() as i32;
		loop {
			let val2 = {
				let c_opt = input_chars.peek();
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
			input_chars.next();
		}
		Some(NumVal::from_i32(val))
	}
}

#[test]
fn test_add() {
	let nv0 = NumVal::from_i32(123);
	assert_eq!("123", nv0.to_string());

	let nv1 = NumVal::from_i32(4567);
	assert_eq!("4567", nv1.to_string());

	let nv2 = NumVal::add(nv0, nv1);
	assert_eq!("4690", nv2.to_string());

	let nv0 = NumVal::from_i32(-45);
	assert_eq!("-45", nv0.to_string());

	assert_eq!("3",  NumVal::add(NumVal::from_i32(1),  NumVal::from_i32(2)).to_string());
	assert_eq!("-3", NumVal::add(NumVal::from_i32(-1), NumVal::from_i32(-2)).to_string());
	assert_eq!("1",  NumVal::add(NumVal::from_i32(-1), NumVal::from_i32(2)).to_string());
	assert_eq!("-1", NumVal::add(NumVal::from_i32(1),  NumVal::from_i32(-2)).to_string());
}

#[test]
fn test_sub() {
	assert_eq!("1",  NumVal::sub(NumVal::from_i32(2),  NumVal::from_i32(1)).to_string());
	assert_eq!("-1", NumVal::sub(NumVal::from_i32(-2), NumVal::from_i32(-1)).to_string());
	assert_eq!("-3", NumVal::sub(NumVal::from_i32(-2), NumVal::from_i32(1)).to_string());
	assert_eq!("3",  NumVal::sub(NumVal::from_i32(2),  NumVal::from_i32(-1)).to_string());

	assert_eq!("-1", NumVal::sub(NumVal::from_i32(1),  NumVal::from_i32(2)).to_string());
	assert_eq!("1",  NumVal::sub(NumVal::from_i32(-1), NumVal::from_i32(-2)).to_string());
	assert_eq!("-3", NumVal::sub(NumVal::from_i32(-1), NumVal::from_i32(2)).to_string());
	assert_eq!("3",  NumVal::sub(NumVal::from_i32(1),  NumVal::from_i32(-2)).to_string());
}

#[test]
fn test_mul() {
	assert_eq!("15",  NumVal::mul(NumVal::from_i32(3),  NumVal::from_i32(5)).to_string());
	assert_eq!("150",  NumVal::mul(NumVal::from_i32(30),  NumVal::from_i32(5)).to_string());
	assert_eq!("1500",  NumVal::mul(NumVal::from_i32(30),  NumVal::from_i32(50)).to_string());
	assert_eq!("9801",  NumVal::mul(NumVal::from_i32(99),  NumVal::from_i32(99)).to_string());

	assert_eq!("28",  NumVal::mul(NumVal::from_i32(4),  NumVal::from_i32(7)).to_string());
	assert_eq!("-28",  NumVal::mul(NumVal::from_i32(-4),  NumVal::from_i32(7)).to_string());
	assert_eq!("-28",  NumVal::mul(NumVal::from_i32(4),  NumVal::from_i32(-7)).to_string());
	assert_eq!("28",  NumVal::mul(NumVal::from_i32(-4),  NumVal::from_i32(-7)).to_string());
}

#[test]
fn test_div() {
	assert_eq!("15.625",  NumVal::div(NumVal::from_i32(1000),  NumVal::from_i32(64)).to_string());
	assert_eq!("-15.625",  NumVal::div(NumVal::from_i32(-1000),  NumVal::from_i32(64)).to_string());
	assert_eq!("-15.625",  NumVal::div(NumVal::from_i32(1000),  NumVal::from_i32(-64)).to_string());
	assert_eq!("15.625",  NumVal::div(NumVal::from_i32(-1000),  NumVal::from_i32(-64)).to_string());
}

#[test]
fn test_div_mod() {
	assert_eq!("4",  NumVal::div_mod(NumVal::from_i32(100),  NumVal::from_i32(48)).to_string());
	assert_eq!("4",  NumVal::div_mod(NumVal::from_i32(-100),  NumVal::from_i32(48)).to_string());
	assert_eq!("-4",  NumVal::div_mod(NumVal::from_i32(100),  NumVal::from_i32(-48)).to_string());
	assert_eq!("-4",  NumVal::div_mod(NumVal::from_i32(-100),  NumVal::from_i32(-48)).to_string());
}
