#[macro_use]
extern crate lazy_static;

use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;

extern crate libc;
use libc::{c_int, sighandler_t, signal};
use libc::{SIGHUP, SIGINT, SIGTERM};

#[derive(Copy, Clone, Debug)]
pub enum Signal {
	SIGHUP,
	SIGINT,
	SIGTERM,
}

static MASK: AtomicUsize = AtomicUsize::new(0);
lazy_static! {
	static ref CVAR: Condvar = Condvar::new();
	static ref MUTEX: Mutex<()> = Mutex::new(());
}

pub fn handle<F>(signals: &'static [Signal], handler: F)
where
	F: Fn(&[Signal]) + 'static + Send,
{
	for &signal in signals.iter() {
		unsafe {
			set_handler(signal);
		}
	}

	thread::spawn(move || {
		let mut sigs = Vec::new();
		loop {
			let mask = MASK.load(Ordering::Relaxed);
			if mask == 0 {
				let _ = CVAR.wait(MUTEX.lock().unwrap());
				thread::yield_now();

				continue;
			}

			sigs.clear();

			if mask & 1 != 0 {
				sigs.push(Signal::SIGHUP);
			}
			if mask & 2 != 0 {
				sigs.push(Signal::SIGINT);
			}
			if mask & 1024 != 0 {
				sigs.push(Signal::SIGTERM);
			}

			MASK.store(0, Ordering::Relaxed);

			handler(&sigs);
		}
	});
}

extern "C" fn handler(sig: c_int) {
	let mask = match sig {
		SIGHUP => 1,
		SIGINT => 2,
		SIGTERM => 1024,
		_ => return,
	};

	loop {
		let prev_mask = MASK.load(Ordering::Relaxed);
		let new_mask = prev_mask | mask;
		if MASK.compare_and_swap(prev_mask, new_mask, Ordering::Relaxed) == new_mask {
			break;
		}
	}

	CVAR.notify_all();
}

#[inline]
unsafe fn set_handler(sig: Signal) {
	signal(
		match sig {
			Signal::SIGHUP => SIGHUP,
			Signal::SIGINT => SIGINT,
			Signal::SIGTERM => SIGTERM,
		},
		mem::transmute::<_, sighandler_t>(handler as extern "C" fn(_)),
	);
}
