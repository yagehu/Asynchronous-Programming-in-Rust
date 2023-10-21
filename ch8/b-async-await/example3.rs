use std::{
    thread,
    time::Duration,
    time::Instant,
};

mod http;
mod future;

use future::*;
use crate::http::Http;

coro fn read_request(i: usize) {
    let path = format!("/{}/HelloWorld{i}", i * 1000);
    let txt = Http::get(&path).wait;
    println!("{txt}");
}

coro fn async_main() {
    println!("Program starting");
    let mut futures = vec![];

    for i in 0..5 {
        futures.push(read_request(i));
    }

    future::join_all(futures).wait;
}


fn main() {
    let start = Instant::now();
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::NotReady => {
                thread::sleep(Duration::from_millis(200));
            }

            PollState::Ready(_) => break,
        }
    }

    println!("\nELAPSED TIME: {}", start.elapsed().as_secs_f32());
}
