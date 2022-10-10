mod utils;

use itertools::izip;
use anyhow::{Result, Error};
use clap::Parser;
use serde_derive::{Serialize, Deserialize};

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
#[clap(author, version, about, 
	override_usage = "mepower > FILE.json; mepower | jq",
	long_about = r#"Central Maine Power has a 1990's-esque portal for viewing power outage information.
	This utility scrapes the site and returns neline-delimited JSON (ndjson/jsonlines)
	to stdout with street-level outage and restoration information."#
)]
struct Args {
}

fn main() -> Result<(), Error> {
	
	let _ = Args::parse();
	
	let doc = utils::get_and_parse(format!("{}/CMP.html", URL_BASE).as_str());
	
	let ts_sel = scraper::Selector::parse(TIMESTAMP_SELECTOR).unwrap();
	let mut ts = doc.select(&ts_sel).into_iter().map(|x| x.inner_html());
	let update = ts.next().unwrap().replace("Update: ", "");
	
	let counties_selector = scraper::Selector::parse(TD_1_A).unwrap();
	let counties: Vec<String> = doc.select(&counties_selector).into_iter().map(|x| x.inner_html()).collect();
	
	if counties.is_empty() {
		println!(r#"{{"outage_update":"{}","message":"No outage data found."}}"#, update);
	} else {
		let county_urls = doc.select(&counties_selector).into_iter().map(|x| x.value().attr("href").unwrap());
		
		let county_totals_selector = scraper::Selector::parse(TD_2).unwrap();
		let county_total_customers = doc.select(&county_totals_selector).into_iter().map(|x| x.inner_html());
		
		let county_out_selector = scraper::Selector::parse(TD_3).unwrap();
		let county_total_out = doc.select(&county_out_selector).into_iter().map(|x| x.inner_html());
		
		let mut outages: Vec<OutageRecord> = Vec::new();
		
		izip!(counties, county_urls, county_total_customers, county_total_out)
		.for_each(| (county, county_url, county_total, county_out) | {
			
			let doc = utils::get_and_parse(format!("{}/{}", URL_BASE, county_url).as_str());
			
			let munis_selector = scraper::Selector::parse(TD_1_A).unwrap();
			let munis: Vec<String> = doc.select(&munis_selector).into_iter().map(|x| x.inner_html()).collect();
			let munis_urls = doc.select(&munis_selector).into_iter().map(|x| x.value().attr("href").unwrap());
			
			let munis_total_customers_selector = scraper::Selector::parse(TD_2).unwrap();
			let munis_total_customers = doc.select(&munis_total_customers_selector).into_iter().map(|x| x.inner_html());
			
			let munis_impacted_selector = scraper::Selector::parse(TD_3).unwrap();
			let munis_impacted = doc.select(&munis_impacted_selector).into_iter().map(|x| x.inner_html());
			
			izip!(munis, munis_urls, munis_total_customers, munis_impacted)
			.for_each(| (muni, muni_url, muni_total, muni_out) | {
				
				let doc = utils::get_and_parse(format!("{}/{}", URL_BASE, muni_url).as_str());
				let streets = utils::html_nodes(doc.to_owned(), TD_1);

				let street_selector = scraper::Selector::parse(TD_1).unwrap();
				let streets = doc.select(&street_selector).into_iter().map(|x| x.inner_html());
				
				let street_impacted_selector = scraper::Selector::parse(TD_3).unwrap();
				let streets_impacted = doc.select(&street_impacted_selector).into_iter().map(|x| x.inner_html());
				
				let estimated_restoration_selector = scraper::Selector::parse(TD_4).unwrap();
				let estimated_restoration = doc.select(&estimated_restoration_selector).into_iter().map(|x| x.inner_html());
				
				izip!(streets, streets_impacted, estimated_restoration)
				.for_each(| (street, street_out, street_restoration) | {
					
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
