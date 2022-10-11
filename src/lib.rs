use itertools::izip;
use serde_derive::{Deserialize, Serialize};
use chrono::prelude::*;

use anyhow::{Error, Result};

const TIMESTAMP_SELECTOR: &str = "body > p[align='right']";
const TD_1_A: &str = "td:nth-child(1) > a";
const TD_1: &str = "td:nth-child(1)";
const TD_2: &str = "td:nth-child(2)";
const TD_3: &str = "td:nth-child(3)";
const TD_4: &str = "td:nth-child(4)";

// pub(crate) const URL_BASE: &str = "https://tycho.local/ecmp.cmpco.com/OutageReports";

const URL_BASE: &str = "https://ecmp.cmpco.com/OutageReports";

/// This holds each outage record that we serialize to JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct OutageRecord {
	pub outage_update: String,
	pub county: Option<String>,
	pub county_total: Option<String>,
	pub county_out: Option<String>,
	pub muni: Option<String>,
	pub muni_total: Option<String>,
	pub muni_out: Option<String>,
	pub street: Option<String>,
	pub street_out: Option<String>,
	pub street_restoration: Option<String>,
	pub message: String,
}

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36";

// way less verbose then repeating the reqwest idiom
fn get(url: &str) -> Result<String, Error> {
	let client = reqwest::blocking::Client::new();

	Ok(
		client
		.get(url)
		.header("User-Agent", UA)
		.send()?
		.text()?
	)

}

// get and parse in one step
pub fn get_and_parse(url: &str) -> Result<scraper::Html, Error> {
	Ok(scraper::Html::parse_document(&get(url)?))
}

// retrieve the text of the specified HTML nodes
pub fn html_nodes(doc: scraper::Html, selector: &str) -> Vec<String> {
	let sel = scraper::Selector::parse(selector).unwrap();
	doc.select(&sel).into_iter().map(|x| x.inner_html()).collect()
}

// retrieve the text of the specified attribute at the specified nodes
pub fn html_nodes_attr(doc: scraper::Html, selector: &str, attr: &str) -> Vec<String> {
	let sel = scraper::Selector::parse(selector).unwrap();
	doc.select(&sel).into_iter().map(|x| x.value().attr(attr).unwrap().to_string()).collect()
}

pub fn get_outages() -> Vec<OutageRecord> {

	let mut outages: Vec<OutageRecord> = Vec::new();

	// get the main page

	let doc = get_and_parse(format!("{}/CMP.html", URL_BASE).as_str());

	if doc.is_err() {

		let utc: DateTime<Utc> = Utc::now();
		let err_rec = OutageRecord {
			outage_update: utc.format("%b %e, %Y %T").to_string(), // Oct 11, 2022 12:40 AM"
			county: None,
			county_total: None,
			county_out: None,
			muni: None,
			muni_total: None,
			muni_out: None,
			street: None,
			street_out: None,
			street_restoration: None,
			message: String::from("CMP Scraping Error"),
		};

		outages.push(err_rec);

		return outages;

	}

	let doc = doc.unwrap();

	let ts = html_nodes(doc.to_owned(), TIMESTAMP_SELECTOR);
	let update = ts[0].replace("Update: ", "");

	let counties = html_nodes(doc.to_owned(), TD_1_A);
	
	// if the page isn't empty traverse counties -> munis
	if counties.is_empty() {
		println!(r#"{{"outage_update":"{}","message":"No outage data found."}}"#, update);
	} else {
		let county_urls = html_nodes_attr(doc.to_owned(), TD_1_A, "href");
		let county_total_customers = html_nodes(doc.to_owned(), TD_2);
		let county_total_out = html_nodes(doc, TD_3);

		izip!(counties, county_urls, county_total_customers, county_total_out).for_each(|(county, county_url, county_total, county_out)| {
			let doc = get_and_parse(format!("{}/{}", URL_BASE, county_url).as_str()).unwrap();

			let munis = html_nodes(doc.to_owned(), TD_1_A);
			let munis_urls = html_nodes_attr(doc.to_owned(), TD_1_A, "href");
			let munis_total_customers = html_nodes(doc.to_owned(), TD_2);
			let munis_impacted = html_nodes(doc, TD_3);

			izip!(munis, munis_urls, munis_total_customers, munis_impacted).for_each(|(muni, muni_url, muni_total, muni_out)| {
				let doc = get_and_parse(format!("{}/{}", URL_BASE, muni_url).as_str()).unwrap();

				let streets = html_nodes(doc.to_owned(), TD_1);
				let streets_impacted = html_nodes(doc.to_owned(), TD_3);
				let estimated_restoration = html_nodes(doc, TD_4);

				izip!(streets, streets_impacted, estimated_restoration).for_each(|(street, street_out, street_restoration)| {
					let rec = OutageRecord {
						outage_update: update.to_owned(),
						county: Some(titlecase::titlecase(&county)),
						county_total: Some(county_total.to_owned()),
						county_out: Some(county_out.to_owned()),
						muni: Some(titlecase::titlecase(&muni)),
						muni_total: Some(muni_total.to_owned()),
						muni_out: Some(muni_out.to_owned()),
						street: Some(titlecase::titlecase(&street)),
						street_out: Some(street_out),
						street_restoration: Some(street_restoration),
						message: String::from(""),
					};

					outages.push(rec);
				});
			});
		});
	}
	outages
}