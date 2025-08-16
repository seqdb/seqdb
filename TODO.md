# TODO

- __crates__
  - _seqdb_
    - remove `identifier` enum
      - and store the index of region in the region
    - improve durability
    - move example to tests
  - _vecdb_
    - make serde an optional feature
    - eager: ema support
    - move example to tests
    - create import options
    - try `mmap.advise(x)` and see how it impacts things (especially in threads using the mmap in conflicting ways)
    - support other compression algos such as lz4 and zstd
- __docs__
  - benchmark using fjall bench
