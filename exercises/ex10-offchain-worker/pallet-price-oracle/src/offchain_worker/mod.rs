mod error;
use core::str::FromStr;

use error::OffchainWorkerError;

use crate::{Call, Config, Pallet, String, Vec};

use frame_support::sp_runtime::offchain::http;
use frame_support::log;


use serde::Deserialize;
use serde_json::{Value,Number};
use sp_arithmetic::{FixedI64, FixedPointNumber};
use frame_support::sp_io::offchain::{timestamp};
use sp_core::offchain::{Duration};
use frame_system::offchain::{SubmitTransaction};

#[derive(Debug, Deserialize)]
struct PairBuyPrice {
	base: String,
	currency: String,
	amount: String,
}

#[derive(Debug, Deserialize)]
struct CoinbaseResponseBody {
	data: PairBuyPrice,
}

pub(crate) fn fetch_btc_price() -> Result<FixedI64, OffchainWorkerError> {
	// TODO:
	// - do an http get request to `"https://api.coinbase.com/v2/prices/BTC-USD/buy`
	// - extract the price form the response body
	// - convert it to `FixedI64` before returning it
    let deadline = timestamp().add(Duration::from_millis(2_000));
    let request = http::Request::get("https://api.coinbase.com/v2/prices/BTC-USD/buy");
    let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
    let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
    if response.code != 200 {
	log::warn!("Unexpected status code: {}", response.code);
	return Err(OffchainWorkerError::Request(http::Error::Unknown));
    }
    let body = response.body().collect::<Vec<u8>>();
    let body_json : Value = serde_json::from_slice(&body)?;
    if let  Some(Value::String(price)) = body_json.get("data").and_then(|v| v.get("amount")) {
	match <f64 as FromStr>::from_str(price) {
	    Ok(f64_price) => Ok(f64_to_fixed_i64(f64_price)),
	    Err(e) => Err(OffchainWorkerError::ParsePrice(e)), 
	}
    } else {
	Err(OffchainWorkerError::Request(http::Error::Unknown))
    }
}

impl<T: Config> Pallet<T> {
	pub(crate) fn fetch_btc_price_and_send_unsigned_transaction() -> Result<(), String> {
	    // Todo: call `fetch_btc_price` and use the return to submit an unsigned transaction
	    // containing a call to `set_btc_price`
	    let btc_price = fetch_btc_price().unwrap();
	    let call  = Call::set_btc_price {btc_price};
	    SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).unwrap();
	    Ok(())
	}
}

// FixedI64::from_float is only available in `std` mode.
// This is a copy-paste of it's implementation, which as shown by the test bellow,
// works just fine for the values and precision we are working with
//
// Feel free to use!
fn f64_to_fixed_i64(n: f64) -> FixedI64 {
	FixedI64::from_inner((n * (<FixedI64 as FixedPointNumber>::DIV as f64)) as i64)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn f64_to_fixed_i64_ok() {
		let mut x: f64 = 0.00;
		while x < 100_000.00 {
			assert_eq!(FixedI64::from_float(x), f64_to_fixed_i64(x));
			x += 0.01;
		}
	}
}
