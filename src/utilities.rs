use std::{fs, io};
use std::fs::DirEntry;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};

pub(crate) const SOURCE_DIR: &'static str = "file-source";
pub(crate) const STATUS_DIR: &'static str = "file-source/file-status";
pub(crate) const SIMULATION_DIR: &'static str = "src/simulation";

#[derive(Debug, Clone)]
pub struct FileEntry {
    /// file's name without the extension string.
    pub file_name: String,

    /// The part after the first "."
    pub file_extension: String,

    /// The full path of the file.
    pub full_path: String,
}

impl FileEntry {
    pub fn new(entry: &DirEntry) -> Self {
        let full_path = entry.path().into_os_string().into_string().unwrap();
        let (_, extension) = full_path.split_once(".").unwrap();
        let file_name: &str = full_path.split("/").last().unwrap();
        let file_name = file_name.split(".").take(1).collect::<String>();
        FileEntry {
            file_name,
            file_extension: String::from(extension),
            full_path
        }
    }
}

pub fn get_ready_files() -> Result<Vec<FileEntry>, std::io::Error> {
    let files = walk_directory(STATUS_DIR, &|x: &str| x.ends_with(".ready"))?;

    Ok(files)
}

pub fn get_done_files() -> Result<Vec<FileEntry>, std::io::Error>  {
    let files = walk_directory(SOURCE_DIR, &|x: &str| x.ends_with(".done"))?;

    Ok(files)
}

pub fn walk_directory(path: &str, fn_filter: impl Fn(&str) -> bool) -> Result<Vec<FileEntry>, std::io::Error> {
    let entries = fs::read_dir(&path)?;

    let mut files: Vec<FileEntry> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            let file_name = entry.file_name();
            if fn_filter(file_name.to_str().unwrap()) {
                files.push(FileEntry::new(&entry));
            }
        }
    }

    Ok(files)
}

type Job<T> = Box<dyn FnOnce() -> T + Send + 'static>;

struct Worker {
    id: u8,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new<T>(id: u8,
           sender: Sender<T>,
           pool_receiver: Arc<Mutex<Receiver<Job<T>>>>) -> Worker
           where T: Send + 'static {
        let thread = thread::spawn(move || loop {
            let job = pool_receiver.lock().unwrap().recv();
            if job.is_err() {
                println!("Failed to acquire a job, terminating.");
                return;
            }

            let job = job.unwrap();
            let res: T = job();
            sender.send(res).expect("Failed to send a result back to the pool");
        });
        Worker { id, thread }
    }
}

pub struct ThreadPool<T: Send + 'static> {
    /// specifies the number of threads.
    workers: Vec<Worker>,

    result_receiver: Receiver<T>,

    job_sender: Sender<Job<T>>,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new(n: u8) -> Self {
        let mut workers = Vec::with_capacity(n.into());
        let (sender, receiver) = mpsc::channel();
        let (sender_v, receiver_v) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..n {
            workers.push(Worker::new(id, sender_v.clone(), receiver.clone()));
        }
        ThreadPool {
            workers,
            result_receiver: receiver_v,
            job_sender: sender,
        }
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() -> T + Send + 'static
    {
        let job = Box::new(f);
        self.job_sender.send(job).expect("Failed to send a job");
    }

    pub fn collect_results(&self, n: usize) -> Vec<T>
        where T: Sized
    {
        let mut collected_results = Vec::with_capacity(n);

        while collected_results.len() < n {
            let res = self.result_receiver.recv().expect("Failed to receive a result");
            collected_results.push(res);
        } 

        collected_results
    }

    /// TODO: Need to make a graceful shutdown implemented.
    /// ALso, need cancellation tokens.
    pub fn shutdown(&self) {
        
    }
}