use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;

enum JobKind {
    Fizz,
    Buzz,
    FizzBuzz,
    Number,
}

// Problem statement can be found here:
// https://leetcode.com/problems/fizz-buzz-multithreaded/description/
struct FizzBuzz {
    n: u16,
    counter: Mutex<u16>,
    cvar: Condvar,
}

impl FizzBuzz {
    fn new(n: u16) -> Self {
        FizzBuzz {
            n,
            counter: Mutex::new(1),
            cvar: Condvar::new(),
        }
    }

    fn do_work(&self, job_kind: JobKind) {
        let mut counter = self.counter.lock().unwrap();

        // each thread should process every number from 1 to N
        // it will only do work for numbers that match the condition associated with the JobKind
        while *counter <= self.n {
            // see if the current number is applicable to the job type and print the relevant
            // statement, if so.
            if self.process_req(&job_kind, &mut counter) {
                // increment the counter if this thread processed the current number
                *counter += 1;
                // don't notify other threads about the update yet, instead see if this job can
                // process the number again
                continue;
            } else {
                // if this thread cannot process the current number, wake up the other threads
                self.cvar.notify_all();
            }

            // unlock the counter and wait for another thread to process the current number
            counter = self.cvar.wait(counter).unwrap();
        }

        // once this thread sees the last number, wake the rest up, so they can exit
        self.cvar.notify_all();
    }

    fn process_req(&self, kind: &JobKind, counter: &mut MutexGuard<u16>) -> bool {
        let print_number = || println!("{}", **counter);
        let (should_run, operation): (bool, &dyn Fn()) = match kind {
            JobKind::Fizz => (**counter % 3 == 0 && **counter % 5 != 0, &|| {
                println!("fizz")
            }),
            JobKind::Buzz => (**counter % 3 != 0 && **counter % 5 == 0, &|| {
                println!("buzz")
            }),
            JobKind::FizzBuzz => (**counter % 3 == 0 && **counter % 5 == 0, &|| {
                println!("fizzbuzz")
            }),
            JobKind::Number => (**counter % 3 != 0 && **counter % 5 != 0, &print_number),
        };

        if should_run {
            // returning the operation out of this fn requires Boxing the "operations" to avoid
            // reference lifetime issues (due to counter being borrowed). Not sure if there's a
            // more crabby way of dealing with this situation without the side effect here.
            operation();
        }
        should_run
    }
}

fn main() {
    let fizz_buzz = Arc::new(FizzBuzz::new(15));

    let job_kinds = vec![
        JobKind::Fizz,
        JobKind::Buzz,
        JobKind::FizzBuzz,
        JobKind::Number,
    ];
    let mut handles = Vec::new();

    for job_kind in job_kinds {
        let atomic_ref = Arc::clone(&fizz_buzz);
        let handle = thread::spawn(move || atomic_ref.do_work(job_kind));
        handles.push(handle);
    }

    handles.into_iter().for_each(|h| h.join().unwrap());
}
