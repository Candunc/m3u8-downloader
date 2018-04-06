extern crate hyper;
extern crate hyper_rustls;
extern crate url;

use std::env;
use std::process;

mod download;

fn main() {
	let arguments: Vec<String> = env::args().collect();
	if arguments.len() < 3 {
		println!("Usage: m3u8-downloader [url] [filename]");
		process::exit(1);
	}

	download::m3u8(&arguments[1], 0, &arguments[2]);
}
