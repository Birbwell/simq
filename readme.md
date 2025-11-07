A job queue with the goal of being as simple of a queue as possible, and having the easiest setup compared to other queues. Made this mainly just as a "how to make/publish a crate" project.

This job queue is written purely with the standard library; it is not written with tokio/async-std in mind. I'm not sure if this poses an issue when using tokio/async-std, but it shouldn't cause anything based on what I know.

# Installation

`cargo add simq`

# Features

- Easy-to-use function-based job runner. Just one line of code and it'll be up and running.
- Customizeable callback functions, so any extra data processing is a breeze.
    - Good for writing logs, adding the result to a database, or if the base job function is immutable.

# Usage

```rust
fn worker_function((param1, param2): (Type1, Type2)) -> Type3 {
    ..
}

fn some_function() {
    let sc: SimqChannel<(Type1, Type2), Type3> = SimqBuilder::register(5, worker_function)
        .and_then(|res: &Type3| tracing::info!("Produced {res:?}"))
        .spawn();

    let result_id = sc.send((val1, val2)).unwrap();

    // wait some time

    // `None` means the result is not ready, or already been retrieved
    let result: Option<Type3> = sc.get(result_id);
}
```

This creates a SimqWorker that is able to create 5 threads that run `worker_function`. The parameters `val1` and `val2` are sent to be processed, returning `result_id`. After some time, the results of the job can be retrieved with `wc.get(result_id)`.

# Known Issues

- Not really an issue, but limitations of generics made it so you have you define your functions using tuples, if there is more than one parameter. So a function can be `f(a)` or `f((a, b))`, but not `f(a, b)`.

- If a job panics, no result will be available, ever. You'll get the panic message in the console, but nothing else. Idk if this should be addressed, it may be even desired, but I thought I should point it out.

- The idea is that the queue will stop processing new tasks once the SimqChannel is dropped. This has largely been accomplished, and existing tasks in the queue will continue to be processed until there are none left, however it will NOT continue running them if the main process ends, nor will the main process wait for them to finish.

- the result_id generator does not wrap around. Though it is unlikely for any process to reach `usize::MAX`, it could potentially cause issues if someone uses this crate for an application that would start `usize::MAX` tasks.

Other than that, just remember the queue is very basic. It may have worked for my use cases, but if you need something more complex, this may not have the features you need.
