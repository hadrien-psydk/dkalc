use big_dec;
use big_dec::BigDec;

pub fn bd_zero(_: BigDec) -> Result<BigDec, big_dec::Error> {
	Ok(BigDec::zero())
}

pub fn bd_same(arg: BigDec) -> Result<BigDec, big_dec::Error> {
	Ok(arg)
}
