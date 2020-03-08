use moos_sys::{resolve, to_app, MoosApp, MoosInterface, MoosMessageData};
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use std::{mem, path, sync, thread};

pub trait Reporter {
    fn report(&self) -> String;
}

pub trait Communicator {
    fn publish(&mut self) -> HashMap<String, MoosMessageData>;
    fn mail(&mut self, mail: HashMap<String, MoosMessageData>);
    fn iterate(&mut self) {}
}

pub trait Runner: Send {
    fn iterate(&mut self); // -> thread::JoinHandle<()>;
}

pub struct App<'a> {
    base_app: *mut MoosApp,
    name: &'a str,
    mission: path::PathBuf,
    subscriptions: Vec<&'a str>,
    runner_object: Arc<Mutex<Box<dyn Runner>>>,
    communicator_object: Box<dyn Communicator>,
    reporter_object: Box<dyn Reporter>,
    handle: Option<thread::JoinHandle<()>>,
}

impl<'a> App<'a> {
    pub fn new<T>(
        name: &'a str,
        mission: &path::Path,
        subcriptions: Vec<&'a str>,
        object: Box<T>,
    ) -> Self
    where
        T: 'static + Runner + Communicator + Clone + Reporter,
    {
        App {
            base_app: moos_sys::MoosApp::new::<App>(),
            name,
            mission: path::PathBuf::from(mission),
            subscriptions: subcriptions,
            runner_object: Arc::new(Mutex::new(object.clone())),
            communicator_object: object.clone(),
            reporter_object: object,
            handle: None,
        }
    }

    pub fn start(&mut self) {
        let a = sync::Arc::clone(&self.runner_object);

        self.handle = Some(thread::spawn(move || loop {
            a.lock().unwrap().iterate();
        }));

        self.run(self.name, &self.mission.clone());
    }
}

impl<'a> MoosInterface for App<'a> {
    extern "C" fn iterate(app_ptr: *mut c_void) -> bool {
        let (this_app, base_app) = resolve::<App>(app_ptr);

        this_app.communicator_object.iterate();

        for (key, value) in this_app.communicator_object.publish() {
            base_app.notify(&key, &value);
        }

        true
    }

    extern "C" fn on_connect_to_server(app_ptr: *mut c_void) -> bool {
        let (this_app, base_app) = resolve::<App>(app_ptr);

        for key in &this_app.subscriptions {
            base_app.register(key, 0.0);
        }

        true
    }

    extern "C" fn on_start_up(_app_ptr: *mut c_void) -> bool {
        true
    }

    fn on_new_mail(app_ptr: *mut c_void, data: HashMap<String, MoosMessageData>) -> bool {
        let (this_app, _base_app) = resolve::<App>(app_ptr);

        this_app.communicator_object.mail(data);

        true
    }

    fn base_app(&mut self) -> &'static mut MoosApp {
        to_app(self.base_app)
    }

    extern "C" fn on_build_report(app_ptr: *mut c_void) -> *const i8 {
        let (this_app, _base_app) = resolve::<App>(app_ptr);

        let str_report = CString::new(this_app.reporter_object.report()).unwrap();
        // let str_report = CString::new("").unwrap();
        let c_report = str_report.as_ptr();
        mem::forget(str_report);

        c_report
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
