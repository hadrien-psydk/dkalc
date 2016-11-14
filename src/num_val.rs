//use std;

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
}
