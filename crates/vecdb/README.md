# vecdb

A K.I.S.S. index-value storage engine that provides persistent, type-safe vector storage with compression and computation capabilities.

## What is vecdb?

vecdb is a high-level vector storage engine built on [seqdb](../seqdb) that provides persistent vector-like data structures. It supports multiple storage formats and computation strategies for different performance and space requirements.

## Key Features

- **Multiple storage variants**: Raw, compressed, lazy, eager, and computed vectors
- **Advanced compression**: Uses Pcodec for numerical data compression
- **Type safety**: Generic storage with zero-copy access
- **Versioning system**: Change tracking and rollback support
- **Hole management**: Efficient sparse data handling
- **Thread-safe**: Concurrent read operations
