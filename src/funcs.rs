use big_dec;
use big_dec::BigDec;
use token;

pub enum Error {
	CallFailed(big_dec::Error),
	UnknownFunc
}

impl Error {
	pub fn to_string(&self) -> String {
		match *self {
			Error::CallFailed(ref bd_err) => bd_err.to_string(),
			Error::UnknownFunc => "unknown func".to_string()
		}
	}
}

struct FuncDisp {
	name: &'static str,
	bdf: fn(arg: BigDec) -> Result<BigDec, big_dec::Error>
}

pub fn eval_func(name: token::Name, arg: big_dec::BigDec) -> Result<BigDec, Error> {
	let func_disps = [
		FuncDisp { name: "zero", bdf: bd_zero },
		FuncDisp { name: "same", bdf: bd_same },
		FuncDisp { name: "sqrt", bdf: bd_sqrt }
	];
	let name_str = name.to_string();
	for fd in &func_disps {
		if name_str == fd.name {
			match (fd.bdf)(arg) {
				Ok(val) => return Ok(val),
				Err(err) => return Err(Error::CallFailed(err))
			}

		}
	}
	Err(Error::UnknownFunc)
}

fn bd_zero(_: BigDec) -> Result<BigDec, big_dec::Error> {
	Ok(BigDec::zero())
}

fn bd_same(arg: BigDec) -> Result<BigDec, big_dec::Error> {
	Ok(arg)
}

fn bd_sqrt(arg: BigDec) -> Result<BigDec, big_dec::Error> {

	let mut r = BigDec::from_i32(1);
	let two = BigDec::from_i32(2);

	let limit = big_dec::BigDec::max_len() * 2;
	for _ in 0..limit {
		let arg_div_r = try!(BigDec::div(arg, r));
		let r_add_adr = try!(BigDec::add(r, arg_div_r));
		let r_aadr_div2 = try!(BigDec::div(r_add_adr, two));
		if BigDec::compare(r, r_aadr_div2) == 0 {
			break;
		}
		r = r_aadr_div2;

		//println!("{} r: {}", i, r);
	}
	Ok(r)
}

#[test]
fn test_sqrt() {
	let n = BigDec::from_i32(25);
	let res = bd_sqrt(n);
	assert!(res.is_ok());
	assert_eq!(BigDec::from_i32(5), res.unwrap());
}

