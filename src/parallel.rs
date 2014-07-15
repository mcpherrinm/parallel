use std::mem::transmute;
use std::raw::Slice;

/// Run a function in parallel over chunks of an array.
pub fn parallel<'data, Data> (data: &'data mut [Data], parallelism: uint, work: fn(uint, &mut [Data])) {
  let chunk_size = (data.len() + parallelism - 1) / parallelism;
  assert!(chunk_size*parallelism >= data.len());

  let mut workeridx = 0u;
  let (tx, rx) = std::comm::channel();
  for (i, chunk) in data.mut_chunks(chunk_size).enumerate() {
    workeridx += 1;
    let this_tx = tx.clone();
    // We are splitting up the input data into chunks, and sending them to worker tasks.  This
    // requires unsafe code on both the parent and child tasks behalf, since we need to avoid
    // the wrath of the type system.  This is safe since we ensure the parent blocks until all
    // children have completed their processing.
    let raw_chunk: Slice<Data> = unsafe { transmute(chunk) };
    let datap = raw_chunk.data as uint;
    let datalen = raw_chunk.len;
    spawn(proc(){
      let chunk = unsafe{ transmute(Slice{data: transmute::<_, *const uint>(datap), len: datalen})};
      work(i, chunk);
      this_tx.send(());
    });
  }
  for _ in range(0, workeridx) {
    rx.recv(); // we receive one message from each job on its completion
  }
}

#[test]
fn test_parallel() {
  fn foo(worker_id: uint, hunk: &mut [uint]) {
    for t in hunk.mut_iter() {
      *t = worker_id;
    }
  }
  for length in range(1u, 129) {
    for parallelism in range(1, length-1) {
      let mut data = Vec::from_elem(length, 0u);

      let slc = data.as_mut_slice();
      parallel(slc, parallelism, foo);

      for i in range(0u, length) {
        assert!(slc[i] != 0);
      }
    }
  }
}
