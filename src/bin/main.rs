use rsig;
use rsig::Signal;
use std::thread;
use std::process;

fn main() {
	println!("run at process {}", process::id());

	rsig::handle(&[Signal::HUP, Signal::INT, Signal::TERM], |signals| {
		println!("recv {:?}", signals);
	});

	loop {
		thread::yield_now();
	}
}
