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
		FuncDisp { name: "same", bdf: bd_same }
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

pub fn bd_zero(_: BigDec) -> Result<BigDec, big_dec::Error> {
	Ok(BigDec::zero())
}

pub fn bd_same(arg: BigDec) -> Result<BigDec, big_dec::Error> {
	Ok(arg)
}
