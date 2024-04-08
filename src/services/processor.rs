#![feature(negative_impls)]

use std::ffi;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::thread;

use crate::utilities;



/// This metadata is maintained by the main proccessor, and not thread safe.
struct Metadata {
    /// the first file that processing error has occured,
    /// there is no need to process files that are earlier than this file.
    first_error_at: String,
    
    /// Maintains a mapping between the processed file's name,
    /// and the status, which can be "true" if it was succesful,
    /// "false" otherwise.
    processed_files: HashMap<ffi::OsString, bool>,

    /// The maximum number of items that the "processed_files" can hold on to.
    capacity: usize,

    /// This is added to prevent metadata from accessed/shared between threads.
    /// As, doing so would degrade the performance, which is not necessary, as the
    /// main processor can iterate over the results.
    _marker: PhantomData<*const ()>
}

impl Metadata {
    fn new(capacity: usize) -> Self {
        Metadata {
            first_error_at: String::new(),
            processed_files: HashMap::new(),
            capacity: capacity,
            _marker: PhantomData::default()
        }
    }
}



pub fn service_startup(run_interval_sec: usize) {
}