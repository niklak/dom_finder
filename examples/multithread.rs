use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::unbounded;
use once_cell::sync::Lazy;

use dom_finder::{Config, Finder};

const CFG_YAML: &str = r"
name: root
base_path: html
children:
  - name: results
    base_path: div.serp__results div.result
    many: true
    children:
      - name: url
        base_path: h2.result__title > a[href]
        extract: href
      - name: title
        base_path: h2.result__title
        extract: text
      - name: snippet
        base_path: a.result__snippet
        extract: html
        pipeline: [ [ policy_highlight ] ]
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // There is a benefit to reuse the same `Finder` instance, because it keeps all matchers compiled and ready to use.
    // Also pipeline functions ready to use.
    // And of course the benefit is even bigger, when we use `Finder` in multithreaded environment.

    // Setting favorite concurrency number;
    let concurrency: usize = 2;

    // For Finder purpose we need to keep CFG alive to the end of the program. So we use `Lazy` to initialize it.
    static CFG: Lazy<Config> = Lazy::new(|| Config::from_yaml(CFG_YAML).unwrap());
    // Setting up the finder inside `Arc` to be able to clone it later.
    let finder = Arc::new(Finder::new(&CFG)?);

    // Creating a channel to send html pages -- just for testing purposes
    let (tx, rx) = unbounded::<&str>();

    // Sending dummy pages to the channel. it can be any amount of pages, but they must be the same type of markup.
    // For instance, presented config can handle only duckduckgo search results pages and nothing more.
    for _ in 0..10 {
        let html_page = include_str!("../test_data/page_0.html");
        tx.send(html_page)?;
    }
    // dropping sender -- we don't need it anymore
    drop(tx);

    let workers: Vec<usize> = (1..concurrency + 1).into_iter().collect();
    let mut handles: Vec<JoinHandle<()>> = vec![];

    for i in workers {
        let rx = rx.clone();
        let finder = finder.clone();
        let handle = thread::spawn(move || {
            let worker_id = i;
            let mut total = 0;

            while let Ok(html_page) = rx.recv() {
                // Using `Finder` instance to parse, without cloning it
                let _ = finder.parse(&html_page);
                // result is omitted here, but in the normal case it may be passed to another channel,
                // or it may be collected in some storage (database, etc.).
                total += 1;
            }
            println!("worker: {worker_id} processed {total} pages");
            drop(rx)
        });
        handles.push(handle)
    }
    for handle in handles {
        handle.join().unwrap();
    }
    drop(finder);
    Ok(())
}
