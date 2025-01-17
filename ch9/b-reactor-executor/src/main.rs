use std::{
    io::{ErrorKind, Read, Write},
    mem::MaybeUninit,
    thread,
    time::Duration,
};

mod http;
mod future;

// THOUGHTS
//
// Rewrite state machine to be a struct instead (easier to reason about)
//
// Implement the first example by hand
// See if we can create an easy bulld.rs file that rewites very simple
// examples using our own syntax to a state machine to explain what the
// compiler does when it reaches async/await

use future::{Future, PollState};
use mio::Poll;

use crate::http::Http;

// later fn async_main() {
//     println!("Program starting")
//     let mut buffer = String::new()
//     let formatter = Formatter::new(&mut buffer);
//     let http = Http::new();
//     let txt = siesta http.get("/1000/HelloWorld");
//     formatter.format(txt);
//     let txt2 = siesta http.get("500/HelloWorld2");
//     formatter.format(txt2);
//     println!("{}", buffer);
// }

// later fn async_main() {
//     println!("Program starting")

//     let http = Http::new();
//     let txt = siesta http.get("/1000/HelloWorld");
//     println!("{txt}");
//     let txt2 = siesta http.get("500/HelloWorld2");
//     println!("{txt2}");
// }


struct Formatter<'a> {
    buffer: &'a mut String,
}

impl<'a> Formatter<'a> {
    fn new(buffer: &'a mut String) -> Self {
        Self { buffer }
    }
    fn format(&mut self, txt: String) {
        *self.buffer += "---------\n";
        *self.buffer += &txt;
        *self.buffer += "+++++++++\n";
    }
}


struct Coroutine {
    storage: Storage,
    state: State,
}

struct Storage {
    
}

enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            storage: Storage {},
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        match self.state {
            State::Start => {
                println!("Program starting");
                let fut = Box::new(Http::get("/1000/HelloWorld1"));
                self.state = State::Wait1(fut);
                PollState::NotReady
            }
            
            State::Wait1(ref mut fut) => {
                match fut.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/600/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                        PollState::NotReady
                    }
                    
                    PollState::NotReady => PollState::NotReady,
                }
            }
            
            State::Wait2(ref mut fut2) => {
                match fut2.poll() {
                    PollState::Ready(txt2) => {
                        println!("{txt2}");
                        self.state = State::Resolved;
                        PollState::Ready(())
                    }
                    
                    PollState::NotReady => PollState::NotReady,
                }
            }
            
            State::Resolved => panic!("Polled a resolved future"),
        }
    }
}


fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

fn main() {
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::NotReady => {
                println!("NotReady");
                // call executor sleep
                thread::sleep(Duration::from_millis(200));
            }

            PollState::Ready(s) => break s,
        }
    }
}

