use std;

#[derive(Copy,Clone)]
pub struct NumVal {
	val: i32
}

impl NumVal {
	pub fn to_string(&self) -> String {
		format!("{}", self.val)
	}

	pub fn is_zero(&self) -> bool {
		self.val == 0
	}

	pub fn negate(&self) -> NumVal {
		NumVal { val: -self.val }
	}

	pub fn zero() -> NumVal {
		NumVal { val: 0 }
	}

	pub fn add(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val + nv1.val }
	}

	pub fn sub(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val - nv1.val }
	}

	pub fn mul(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val * nv1.val }
	}

	pub fn div(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val / nv1.val }
	}

	pub fn div_mod(nv0: NumVal, nv1: NumVal) -> NumVal {
		NumVal { val: nv0.val % nv1.val }
	}

	pub fn from_i32(val: i32) -> NumVal {
		NumVal { val: val }
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
