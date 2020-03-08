use moos_rust::{App, Communicator, Reporter, Runner};
use moos_sys::MoosMessageData;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{sync, thread, time};

#[derive(Clone)]
struct SimpleApp {
    pubs: Arc<Mutex<HashMap<String, MoosMessageData>>>,
    mail: Arc<Mutex<HashMap<String, MoosMessageData>>>,
    r: sync::Arc<sync::Mutex<Report>>,
}

struct Report {
    a: u32,
    b: u64,
}

impl SimpleApp {
    fn new() -> Self {
        SimpleApp {
            pubs: Arc::new(Mutex::new(Default::default())),
            mail: Arc::new(Mutex::new(Default::default())),
            r: sync::Arc::new(sync::Mutex::new(Report { a: 0, b: 0 })),
        }
    }

    fn execute(&mut self) {
        println!("execute");
    }
}

impl Reporter for SimpleApp {
    fn report(&self) -> &str {
        "My report"
    }
}

impl Communicator for SimpleApp {
    fn publish(&mut self) -> HashMap<String, MoosMessageData> {
        let value = self.pubs.lock().unwrap().clone();

        self.pubs.lock().unwrap().clear();

        value
    }

    fn mail(&mut self, mail: HashMap<String, MoosMessageData>) {
        println!("Received: {:?}", mail);
        *self.mail.lock().unwrap() = mail;
    }
}

impl Runner for SimpleApp {
    fn iterate(&mut self) {
        self.execute();

        thread::sleep(time::Duration::from_millis(5000));

        if let Some(item) = self.mail.lock().unwrap().get("Double") {
            match item {
                MoosMessageData::DOUBLE(d) => self
                    .pubs
                    .lock()
                    .unwrap()
                    .insert("Double".to_string(), MoosMessageData::DOUBLE(d + 1.0)),
                MoosMessageData::STRING(_) => panic!(),
            };
        } else {
            self.pubs
                .lock()
                .unwrap()
                .insert("Double".to_string(), MoosMessageData::DOUBLE(0.0));
        }
    }
}

fn main() {
    let mut root = crate_root::root().unwrap();
    root.push("examples");
    root.push("simple.moos");

    App::new::<SimpleApp>(
        "Simple",
        &root,
        vec!["Double", "String"],
        Box::new(SimpleApp::new()),
    )
    .start();
}
