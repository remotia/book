# Buffer pools

Buffer pools manage memory reuse across processing steps. Memory buffers are stored in pools from where they are extracted and returned once used, avoiding redundant allocation and consistently improving performance.

## Borrower/redeemer flow

A *borrower* processor pulls a memory buffer from the selected pool, if any is available, and inserts it into the current DTO. Subsequent processors fill the buffer with data. A *redeemer* processor sends the buffer back to the pool once it is no longer needed by the current processing step. The borrower and redeemer may belong to different components — a buffer can travel across components before being returned to the pool.

The policy when the pool is empty, for instance locking until a buffer is available or dropping the frame, must be decided based on the use case.

## Implementation

Pools are backed by asynchronous message queues, bounded by allocating a fixed amount of buffers at run-time. The number of buffers allocated in each pool tunes how much data each processing step can handle in parallel.

```rust
use remotia::buffers::BuffersPool;

let encoding_pool = BuffersPool::new("encoded_frame", 4);

Pipeline::new()
    .link(
        Component::new()
            .append(encoding_pool.borrower())
            .append(AV1Encoder())
    )
    .link(
        Component::new()
            .append(TCPSender())
            .append(encoding_pool.redeemer())
    )
    .run();
```

The borrower pulls a buffer from the pool (if available) and injects it into the DTO. The redeemer returns the buffer from the DTO to the pool so it can be reused for a new frame, making the encoding process more efficient by avoiding repeated memory allocations.

All buffers should be redeemed from the DTO before it is deallocated at the end of a terminal pipeline, or the pool will empty causing a deadlock. The framework provides debugging facilities to locate such problems.

## Feature flag

Buffer pool types are behind the `buffers` feature flag on the `remotia` crate. See the [crate map](./crate-map.md) for the full type list (`BuffersPool`, `BufferAllocator`, `BufferBorrower`, `BufferRedeemer`, `PoolRegistry`, `#[buffers_map]`).

Browse the [API documentation](https://docs.rs/remotia/latest/remotia/) for the full module reference.
