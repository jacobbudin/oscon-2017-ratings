#[macro_use] extern crate lazy_static;
extern crate prettytable;
extern crate regex;
extern crate reqwest;
extern crate scoped_threadpool;
extern crate scraper;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use regex::Regex;
use scoped_threadpool::Pool;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// OSCON Event
struct Event {
    name: String,
    url: String,
    rating: f32,
    review_count: i8,
}

impl Event {
    /// Retrieve HTML body from OSCON event page
    fn fetch(&self) -> String {
        let mut body = String::new();
        let mut resp = reqwest::get(&self.url).unwrap();
        let _ = resp.read_to_string(&mut body);
        body
    }

    /// Populate structure using OSCON event page
    fn sync(&mut self) {
        lazy_static! {
            static ref RATING_RE: Regex = Regex::new(r"\(([0-9.]+), ([0-9]+) ratings\)").unwrap();
        }

        let body = self.fetch();
        let doc = Html::parse_document(&body);

        let title_selector = Selector::parse("h1").unwrap();
        let title_element = doc.select(&title_selector).next();
        let title_html = match title_element {
            Some(element) => element.inner_html(),
            None => return,
        };
        self.name = title_html;

        let ratings_selector = Selector::parse(".en_grade_average").unwrap();
        let ratings_element = doc.select(&ratings_selector).next();
        let ratings_html = match ratings_element {
            Some(element) => element.inner_html(),
            None => return,
        };

        let caps = match RATING_RE.captures(ratings_html.as_str()) {
            Some(v) => v,
            None => return,
        };

        self.rating = caps.get(1).unwrap().as_str().parse::<f32>().unwrap();
        self.review_count = caps.get(2).unwrap().as_str().parse::<i8>().unwrap();
    }
}

/// Echoes an `Event` vector as a plain-text table
fn output(events: &Vec<Event>) {
    let mut table = Table::new();

    // Add header
    table.add_row(Row::new(vec![Cell::new("Event"), Cell::new("Rating")]));

    for event in events {
        let rating = event.rating.to_string();
        table.add_row(Row::new(vec![Cell::new(event.name.as_str()), Cell::new(rating.as_str())]));
    }

    table.printstd();
}

/// Reads, pulls, and prints OSCON 2017 event ratings
fn main() {
    // Open and read `urls.txt` for event URLs
    let urls_path = Path::new("urls.txt");
    let mut urls_file = match File::open(&urls_path) {
        Err(_) => panic!("couldn't open {:?}", urls_path),
        Ok(file) => file,
    };

    let mut urls_content = String::new();
    let _ = urls_file.read_to_string(&mut urls_content);

    let mut events: Vec<Event> = vec![];

    // Generate boilerplate event structs from event URLs
    for line in urls_content.lines().filter(|&s| !s.is_empty()) {
        events.push(Event {
            name: String::from(""),
            url: String::from(line),
            rating: 0.0,
            review_count: 0
        });
    }

    // Populate `Event`s over four threads
    let mut pool = Pool::new(4);

    pool.scoped(|scope| {
        for event in &mut events {
            scope.execute(move || {
                event.sync();
            });
        }
    });

    // Sort `Event` vector from most highly-rated to least
    events.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());

    // Send output to `stdout`
    output(&events);
}
