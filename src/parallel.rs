use std::mem::transmute;

/// First try, definitely just a test and not the right API
/// work is the closure to invoke on each chunk
pub fn parallel<'data, Data> (data: &'data mut [Data], parallelism: uint, work: fn(uint, &mut [Data])) {
  let mut chunk_size = data.len() / parallelism;
  if chunk_size*parallelism < data.len() { chunk_size += 1 }
  assert!(chunk_size*parallelism >= data.len());

  let mut workeridx = 0u;
  let (tx, rx) = std::comm::channel();
  for chunk in data.mut_chunks(chunk_size) {
    workeridx += 1;
    let thistx = tx.clone();
    // We can't send a &mut [Data] or *Data directly, so send the pointer and
    // length as uints
    let raw_chunk: std::raw::Slice<Data> = unsafe { transmute(chunk) };
    let datap: uint = unsafe { std::mem::transmute(raw_chunk.data)};
    let datalen = raw_chunk.len;
    spawn(proc(){
      let raw_chunk = std::raw::Slice{data: unsafe { transmute::<uint,*const uint>(datap) },
                                      len: datalen};
      let unraw_chunk: &mut [Data] = unsafe { transmute(raw_chunk) };
      work(workeridx, unraw_chunk);
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
  fn foo(worker_id: uint, hunk: &mut [uint]) {
    for t in hunk.mut_iter() {
      *t = worker_id;
    }
  }

  parallel(data.as_mut_slice(), 17, foo);

  println!("data is {}", data.as_slice());

}
