use std;
use std::char;

const FRAC_LEN: usize = 10; // number of digits used for the fractional part
const MAX_LEN: usize = FRAC_LEN + 10;

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
	digits: [u8;MAX_LEN], // little-endian. 1402.658 -> 2,0,4,1,..0
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
		println!("bounds: {} {}", start_at, stop_at);

		if start_at == stop_at {
			ret.push('0');
		}
		else {
			if self.neg {
				ret.push('-');
			}

			for i in (start_at..stop_at).rev() {
				let x = self.digits[i] as u32;
				let c = char::from_u32(48 + x).unwrap();
				ret.push(c);
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

	// add without looking at the negative state of the inputs
	fn add_u(nv0: NumVal, nv1: NumVal) -> NumVal {
		let mut ret = NumVal::zero();
		let mut carry = 0;
		for i in 0..MAX_LEN {
			let z = nv0.digits[i] + nv1.digits[i] + carry;
			carry = z / 10;
			let z2 = z % 10;
			ret.digits[i] = z2;
		}
		ret
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

			let mut z;
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

	pub fn mul(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal::zero()
	}

	pub fn div(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal::zero()
	}

	pub fn div_mod(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal::zero()
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

