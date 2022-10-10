//! # mepower
//!
//! Scrape Central Maine Power's [outage portal](https://ecmp.cmpco.com/OutageReports/CMP.html)
//! and return street-level newline-delimited JSON records.

mod utils;

use anyhow::{Error, Result};
use clap::Parser;
use itertools::izip;
use serde_derive::{Deserialize, Serialize};

const TIMESTAMP_SELECTOR: &str = "body > p[align='right']";
const TD_1_A: &str = "td:nth-child(1) > a";
const TD_1: &str = "td:nth-child(1)";
const TD_2: &str = "td:nth-child(2)";
const TD_3: &str = "td:nth-child(3)";
const TD_4: &str = "td:nth-child(4)";

#[cfg(test)]
pub(crate) const URL_BASE: &str = "https://tycho.local/ecmp.cmpco.com/OutageReports";

#[cfg(not(test))]
pub(crate) const URL_BASE: &str = "https://ecmp.cmpco.com/OutageReports";

/// This holds each outage record that we serialize to JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct OutageRecord {
	pub(crate) outage_update: String,
	pub(crate) county: String,
	pub(crate) county_total: String,
	pub(crate) county_out: String,
	pub(crate) muni: String,
	pub(crate) muni_total: String,
	pub(crate) muni_out: String,
	pub(crate) street: String,
	pub(crate) street_out: String,
	pub(crate) street_restoration: String,
	pub(crate) message: String,
}

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

	// get the main page
	let doc = utils::get_and_parse(format!("{}/CMP.html", URL_BASE).as_str());

	let ts = utils::html_nodes(doc.to_owned(), TIMESTAMP_SELECTOR);
	let update = ts[0].replace("Update: ", "");

	let counties = utils::html_nodes(doc.to_owned(), TD_1_A);

	// if the page isn't empty traverse counties -> munis
	if counties.is_empty() {
		println!(r#"{{"outage_update":"{}","message":"No outage data found."}}"#, update);
	} else {
		let county_urls = utils::html_nodes_attr(doc.to_owned(), TD_1_A, "href");
		let county_total_customers = utils::html_nodes(doc.to_owned(), TD_2);
		let county_total_out = utils::html_nodes(doc, TD_3);

		let mut outages: Vec<OutageRecord> = Vec::new();

		izip!(counties, county_urls, county_total_customers, county_total_out).for_each(|(county, county_url, county_total, county_out)| {
			let doc = utils::get_and_parse(format!("{}/{}", URL_BASE, county_url).as_str());

			let munis = utils::html_nodes(doc.to_owned(), TD_1_A);
			let munis_urls = utils::html_nodes_attr(doc.to_owned(), TD_1_A, "href");
			let munis_total_customers = utils::html_nodes(doc.to_owned(), TD_2);
			let munis_impacted = utils::html_nodes(doc, TD_3);

			izip!(munis, munis_urls, munis_total_customers, munis_impacted).for_each(|(muni, muni_url, muni_total, muni_out)| {
				let doc = utils::get_and_parse(format!("{}/{}", URL_BASE, muni_url).as_str());

				let streets = utils::html_nodes(doc.to_owned(), TD_1);
				let streets_impacted = utils::html_nodes(doc.to_owned(), TD_3);
				let estimated_restoration = utils::html_nodes(doc, TD_4);

				izip!(streets, streets_impacted, estimated_restoration).for_each(|(street, street_out, street_restoration)| {
					let rec = OutageRecord {
						outage_update: update.to_owned(),
						county: titlecase::titlecase(&county),
						county_total: county_total.to_owned(),
						county_out: county_out.to_owned(),
						muni: titlecase::titlecase(&muni),
						muni_total: muni_total.to_owned(),
						muni_out: muni_out.to_owned(),
						street: titlecase::titlecase(&street),
						street_out,
						street_restoration,
						message: String::from(""),
					};

					outages.push(rec);
				});
			});
		});

		outages.into_iter().for_each(|outage| {
			let res = serde_json::to_string(&outage).expect("Error serializing outage info to JSON.");
			println!("{}", res);
		});
	}

	Ok(())
}
