const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36";

fn get(url: &str) -> String {

  let client = reqwest::blocking::Client::new();

	client
    .get(url)
    // .get("https://ecmp.cmpco.com/OutageReports/CMP.html")
    .header("User-Agent", UA)
    .send().expect("Error retrieving CMP Outage URL.")
    .text().expect("Error extracting CMP Outage Data.")

}

pub fn get_and_parse(url: &str) -> scraper::Html {
	scraper::Html::parse_document(&get(url))
}


pub fn html_nodes(doc: scraper::Html, selector: &str) -> Vec<String> {
	let sel = scraper::Selector::parse(selector).unwrap();
	doc.select(&sel).into_iter().map(|x| x.inner_html()).collect()
}
