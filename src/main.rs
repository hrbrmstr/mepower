//! # mepower
//!
//! Scrape Central Maine Power's [outage portal](https://ecmp.cmpco.com/OutageReports/CMP.html)
//! and return street-level newline-delimited JSON records.

mod lib;

use anyhow::{Error, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
	author,
	version,
	about,
	override_usage = "mepower > FILE.json; mepower | jq",
	long_about = r#"Central Maine Power has a 1990's-esque portal for viewing power outage information.
	This utility scrapes the site and returns neline-delimited JSON (ndjson/jsonlines)
	to stdout with street-level outage and restoration information."#
)]
struct Args {}

fn main() -> Result<(), Error> {
	let _ = Args::parse();

	let outages = lib::get_outages();

	outages.into_iter().for_each(|outage| {
		let res = serde_json::to_string(&outage).expect("Error serializing outage info to JSON.");
		println!("{}", res);
	});

	Ok(())

}
