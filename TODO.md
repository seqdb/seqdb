# TODO

- __crates__
  - _seqdb_
    - remove `identifier` enum
      - and store the index of region in the region
    - improve durability
    - try `mmap.advise(x)` and see how it impacts things (especially in threads using the mmap in conflicting ways)
    - move example to tests
  - _vecdb_
    - make serde an optional feature
    - eager: ema support
    - move example to tests
    - create import options
- __docs__
  - benchmark using fjall bench
