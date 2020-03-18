use rsig;
use rsig::Signal;
use std::thread;
use std::process;

fn main() {
	println!("run at process {}", process::id());

	rsig::handle(&[Signal::SIGHUP, Signal::SIGINT, Signal::SIGTERM], |signals| {
		println!("recv {:?}", signals);
	});

	loop {
		thread::yield_now();
	}
}
