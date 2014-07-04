#![feature(overloaded_calls)]
use std::mem::transmute;

struct JobWorker<T> {
  state: T
}

impl<'wut, T: Share+std::fmt::Show, Data:std::fmt::Show> std::ops::Fn<(&'wut mut [Data],), ()> for JobWorker<T> {
  fn call(&self, (args,): (&mut [Data],)) {
    println!("Worker here, workin'! {} on {}", self.state, args[0]);
  }
}

/// First try, probalby not the right API
/// work is the closure to invoke on each chunk
pub fn parallel<'data, Data> (data: &'data mut [Data], parallelism: uint) {
  // For differential testing, this is an entirely safe version that operates serially.
  let mut chunk_size = data.len() / parallelism;
  if chunk_size*parallelism < data.len() { chunk_size += 1 }
  assert!(chunk_size*parallelism >= data.len());

  // hax: this worker stuff needs to be passed in for this to be useful
  let jobber = JobWorker{state: 42u};

  let mut workeridx = 0u;
  let (tx, rx) = std::comm::channel();
  for chunk in data.mut_chunks(chunk_size) {
    workeridx += 1;
    let thistx = tx.clone();
    // We can't send a &mut [Data] directly.
    let raw_chunk: std::raw::Slice<Data> = unsafe { transmute(chunk) };
    let datap: uint = unsafe { std::mem::transmute(raw_chunk.data)};
    let datalen = raw_chunk.len;
    spawn(proc(){
      let raw_chunk = std::raw::Slice{data: unsafe { transmute::<uint,*const Data>(datap) }, len: datalen};
      let unraw_chunk: &mut [Data] = unsafe { transmute(raw_chunk) };
      jobber(unraw_chunk);
      thistx.send(workeridx);
    });
  }
  assert!(workeridx == parallelism);
  for _ in range(0, parallelism) {
    rx.recv(); // we receive one message from each job on its completion
  }
  // After all jobs are finished, we can safely return.
  let mut replycount = 0u;
  println!("{}", workeridx);
}

#[test]
fn test_parallel() {
  let mut data = [0u, ..1024];

  parallel(data.as_mut_slice(), 17);

}
