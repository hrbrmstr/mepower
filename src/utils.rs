//! # Crate utilities
//!
//! Internal helpers to make the code a bit more readable.

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36";

// way less verbose then repeating the reqwest idiom
fn get(url: &str) -> String {
	let client = reqwest::blocking::Client::new();

	client
		.get(url)
		// .get("https://ecmp.cmpco.com/OutageReports/CMP.html")
		.header("User-Agent", UA)
		.send()
		.expect("Error retrieving CMP Outage URL.")
		.text()
		.expect("Error extracting CMP Outage Data.")
}

// get and parse in one step
pub fn get_and_parse(url: &str) -> scraper::Html {
	scraper::Html::parse_document(&get(url))
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
