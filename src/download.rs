use hyper::Client;
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;

use hyper_rustls::TlsClient;

use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::io::stdout;
use std::fs;
use std::fs::File;
use std::string::String;
use std::sync::mpsc::channel;
use std::thread;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use url::Url;

const THREADS: usize = 4;

pub fn m3u8(url: &str, index: usize, name: &str) {
	let parsed_url = Url::parse(url).unwrap();
	let mut base_url = parsed_url.join("a").unwrap().into_string();
	base_url.pop(); // remove the 'a' from above

	let m3u8_metadata = m3u8_to_vector(wget_to_string(url));

	let m3u8_data_url = format!("{}{}",&base_url,&m3u8_metadata[index]);
	let m3u8_data = m3u8_to_vector(wget_to_string(&m3u8_data_url));

	download(base_url,&m3u8_data);
	ffmpeg_concatenate(&m3u8_data,name);
}

fn m3u8_to_vector(data: String) -> Vec<String> {
	let mut vec = Vec::new();

	for line in data.split("\n") {
		if line != "" && line.chars().nth(0).unwrap() != '#' {
			vec.push(String::from(line));
		}
	}
	return vec;
}

fn wget_to_string(url: &str) -> String {
	let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));

	let mut res = client.get(&*url)
		.send().unwrap();

	let mut buffer = String::new();
	let _ = res.read_to_string(&mut buffer);

	if res.status == StatusCode::Ok {
		return buffer;
	} else {
		println!("{}",buffer);
		panic!(format!("Error in wget_to_string: {}",res.status));
	}
}

pub fn wget_to_file(url: String, name: String, client: &Client) -> Result<(), Error> {
	if Path::new(&name).exists() {
		// File exists and was already downloaded.
		Ok(())
	} else {
		let tmp_name = format!("{}.tmp",name);
		//http://stackoverflow.com/a/41451006/1687505
		let mut res = client.get(&*url).send().unwrap();
		let mut file = try!(File::create(&tmp_name));

		let mut buf = [0; 128 * 1024];
		loop {
			let len = match res.read(&mut buf) {
				Ok(0) => break, //End of file reached.
				Ok(len) => len,
				Err(err) => panic!(format!("Error in wget_to_file: {}",err)),
			};
			try!(file.write_all(&buf[..len]));
		}

		// Close the file handle so we can rename it
		drop(file);
		fs::rename(tmp_name,name).unwrap();

		Ok(())
	}
}

fn download(base_url: String, files: &Vec<String>) {
	let mut comm = Vec::new();
	let length = files.len();

	let mut file = File::create("toffmpeg.txt").unwrap();
	for i in 0..length {
		let _ = file.write_all(format!("file '{}'\n",files[i]).as_bytes());
	}

	for _ in 0..THREADS {
		let (slave_tx, main_rx) = channel();
		let (main_tx, slave_rx) = channel();
		comm.push((main_tx,main_rx));

		let base = base_url.clone();

		thread::spawn(move || {
			let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));

			// Ask for work
			slave_tx.send(true).unwrap();
			loop {
				let (status, file) = slave_rx.recv().unwrap();
				if status {
					wget_to_file(
						format!("{}{}",base,file),
						file,
						&client
					).unwrap();
					slave_tx.send(true).unwrap();
				} else {
					break;
				}
			}
		});
	}

	let mut live_threads = THREADS;
	let mut j = 0;
	let empty_string = String::new();

	'outer: loop {
		for &(ref tx,ref rx) in &comm {
			// Try recieve will return an error if the channel is closed or if it is empty - both are useful cases to skip processing
			if !rx.try_recv().is_err() {
				if j < length {
					// Flush stdout https://github.com/rust-lang/rust/issues/23818
					print!("\rProcessing {} of {}",j,length);
					stdout().flush().ok().expect("Could not flush stdout");

					tx.send((true,files[j].clone())).unwrap();
					j += 1;
				} else {
					tx.send((false,empty_string.clone())).unwrap();
					live_threads -= 1;
				}

				if live_threads == 0 {
					break 'outer;
				}
			}
		}
	}

	println!("\rDownload complete.            ");
}

fn ffmpeg_concatenate(files: &Vec<String>,name: &str) {
	// Now concatentate the files and clean
	// https://github.com/rust-lang/rust/issues/30098
	let child = Command::new("ffmpeg")
		.args(&["-loglevel","panic","-hide_banner","-stats","-f","concat","-i","toffmpeg.txt","-c:v","copy","-c:a","copy","-bsf:a","aac_adtstoasc"])
		.arg(name)
		.stdout(Stdio::piped())
		.spawn()
		.unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

	let mut out = child.stdout.unwrap();
	let mut read_buf = [0u8; 64];
	while let Ok(size) = out.read(&mut read_buf) {
		if size == 0 {
			break;
		}
		stdout().write_all(&read_buf).unwrap();
	}

	for value in files {
		let _ = fs::remove_file(value);		
	}

	let _ = fs::remove_file("toffmpeg.txt");
}