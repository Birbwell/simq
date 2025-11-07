use std::collections::HashMap;
use std::sync::mpsc::{RecvTimeoutError, SendError, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

pub struct WorkerChannel<A, B> {
    tx: Sender<(usize, A)>,
    results: Arc<Mutex<HashMap<usize, B>>>,
}

impl<A, B: Clone> WorkerChannel<A, B> {
    /// Send data to the worker, returning the id of the request.
    pub fn send(&self, params: A) -> Result<usize, SendError<(usize, A)>> {
        static ID_COUNTER: Mutex<usize> = Mutex::new(0);
        let next = ID_COUNTER
            .lock()
            .ok()
            .and_then(|mut lock| {
                let t = lock.clone();
                *lock += 1;
                Some(t)
            })
            .unwrap();

        self.tx.send((next, params))?;
        Ok(next)
    }

    /// Get the result of a request, provided the request id. \
    /// The return type must be clonable.
    pub fn get(&self, res_id: usize) -> Option<B> {
        self.results.lock().ok()?.remove(&res_id)
    }
}

/// Job Builder
pub struct SimpBuilder<A, B> {
    func: fn(A) -> B,
    max_threads: usize,
    opt_and_then: Option<fn(&B)>,
    opt_and_then_mut: Option<fn(&mut B)>,
}

impl<A: Send + 'static, B: Send + 'static> SimpBuilder<A, B> {
    /// Register a job, designating the maximum number of parallel threads you wish to run the task
    pub fn register(max_threads: usize, func: fn(A) -> B) -> Self {
        Self {
            func,
            max_threads,
            opt_and_then: None,
            opt_and_then_mut: None,
        }
    }

    /// ### CANNOT BE USED IN CONJUNCTION WITH `and_then_mut`
    /// After producing a result, do something with the result
    /// This is good for additional work, like logging or storing to a database
    pub fn and_then(self, and_then: fn(&B)) -> Self {
        Self {
            opt_and_then: Some(and_then),
            opt_and_then_mut: None,
            ..self
        }
    }

    /// ### CANNOT BE USED IN CONJUNCTION WITH `and_then`
    /// After producing a result, do something with the result (mutable)
    /// Same as the and_then callback, but if you need a mutable reference for some reason
    pub fn and_then_mut(self, and_then_mut: fn(&mut B)) -> Self {
        Self {
            opt_and_then_mut: Some(and_then_mut),
            opt_and_then: None,
            ..self
        }
    }

    /// Launch the worker
    /// The worker will continue accepting and processing data as long as the WorkerChannel is not dropped.
    pub fn spawn(self) -> WorkerChannel<A, B> {
        // Spawn the worker
        let (tx, rx) = channel::<(usize, A)>();
        let result_map = Arc::new(Mutex::new(HashMap::new()));
        let job_result_map = result_map.clone();

        spawn(move || {
            let timeout_dur = std::time::Duration::from_millis(100);
            let sem = Arc::new(Mutex::new(self.max_threads));
            loop {
                match rx.recv_timeout(timeout_dur) {
                    Ok((res_id, params)) => {
                        let thread_result_map = job_result_map.clone();
                        loop {
                            if let Ok(mut lock) = sem.lock()
                                && *lock > 0
                            {
                                *lock -= 1;
                                break;
                            }
                            std::thread::sleep(timeout_dur);
                        }
                        let thread_sem = sem.clone();
                        spawn(move || {
                            let mut res = (self.func)(params);
                            if let Some(and_then) = self.opt_and_then {
                                and_then(&res);
                            } else if let Some(and_then_mut) = self.opt_and_then_mut {
                                and_then_mut(&mut res);
                            }
                            if let Ok(mut lock) = thread_result_map.lock() {
                                lock.insert(res_id, res);
                            }
                            if let Ok(mut lock) = thread_sem.lock() {
                                *lock += 1;
                            }
                        });
                    }
                    Err(e) => {
                        if e == RecvTimeoutError::Timeout {
                            continue;
                        } else {
                            // Sender disconnected; the WorkerChannel was dropped, so we should stop this worker
                            break;
                        }
                    }
                }
            }
        });

        WorkerChannel {
            tx,
            results: result_map,
        }
    }
}
