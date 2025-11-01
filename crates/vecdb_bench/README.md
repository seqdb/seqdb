# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## Benchmark 1

**Test**: 10 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb | vecdb_old | fjall3 | fjall2 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.17 ms | **0.08 ms** | 1.03 s | 644.84 ms | 5.63 ms | 0.15 ms | 2.15 ms |
| **Write** | 17.57 ms<br>569.2 Mo/s<br>4.241 GB/s<br>1.757 ns | **15.12 ms<br>661.6 Mo/s<br>4.929 GB/s<br>1.512 ns** | 15.39 s<br>649.9 Ko/s<br>4.958 MB/s<br>1.539 µs | 12.05 s<br>829.6 Ko/s<br>6.330 MB/s<br>1.205 µs | 5.83 s<br>1.716 Mo/s<br>13.09 MB/s<br>582.7 ns | 7.20 s<br>1.388 Mo/s<br>10.59 MB/s<br>720.3 ns | 4.48 s<br>2.230 Mo/s<br>17.01 MB/s<br>448.4 ns |
| **Linear** | **8.14 ms<br>1228 Mo/s<br>9.151 GB/s<br>0.814 ns** | 37.73 ms<br>265.0 Mo/s<br>1.975 GB/s<br>3.773 ns | 713.72 ms<br>14.01 Mo/s<br>106.9 MB/s<br>71.37 ns | 752.31 ms<br>13.29 Mo/s<br>101.4 MB/s<br>75.23 ns | 390.49 ms<br>25.61 Mo/s<br>195.4 MB/s<br>39.05 ns | 89.30 ms<br>112.0 Mo/s<br>854.4 MB/s<br>8.930 ns | 1.01 s<br>9.934 Mo/s<br>75.79 MB/s<br>100.7 ns |
| **Random** | 0.05 ms<br>2145 Mo/s<br>15.98 GB/s<br>0.466 ns | **0.04 ms<br>2489 Mo/s<br>18.54 GB/s<br>0.402 ns** | 192.57 ms<br>519.3 Ko/s<br>3.962 MB/s<br>1.926 µs | 440.59 ms<br>227.0 Ko/s<br>1.732 MB/s<br>4.406 µs | 113.38 ms<br>882.0 Ko/s<br>6.729 MB/s<br>1.134 µs | 98.85 ms<br>1.012 Mo/s<br>7.718 MB/s<br>988.5 ns | 618.37 ms<br>161.7 Ko/s<br>1.234 MB/s<br>6.184 µs |
| **Random Rayon** | **2.18 ms<br>45.95 Mo/s<br>350.5 MB/s<br>21.76 ns** | 5.59 ms<br>17.89 Mo/s<br>136.5 MB/s<br>55.90 ns | 40.82 ms<br>2.449 Mo/s<br>18.69 MB/s<br>408.2 ns | 177.82 ms<br>562.4 Ko/s<br>4.291 MB/s<br>1.778 µs | 16.25 ms<br>6.153 Mo/s<br>46.94 MB/s<br>162.5 ns | 18.54 ms<br>5.395 Mo/s<br>41.16 MB/s<br>185.4 ns | 157.43 ms<br>635.2 Ko/s<br>4.846 MB/s<br>1.574 µs |
| **Disk Size** | **128.00 MB** | **128.00 MB** | 209.45 MB | 137.11 MB | 514.00 MB | 367.13 MB | 244.54 MB |
---

## Benchmark 2

**Test**: 100 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_old | vecdb | fjall3 | fjall2 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.22 ms | **0.18 ms** | 242.08 ms | 33.29 ms | 5.90 ms | 0.23 ms |
| **Write** | 123.55 ms<br>809.4 Mo/s<br>6.030 GB/s<br>1.236 ns | **102.57 ms<br>975.0 Mo/s<br>7.264 GB/s<br>1.026 ns** | 2m 33.15s<br>653.0 Ko/s<br>4.982 MB/s<br>1.531 µs | 2m 7.42s<br>784.8 Ko/s<br>5.988 MB/s<br>1.274 µs | 1m 51.40s<br>897.7 Ko/s<br>6.849 MB/s<br>1.114 µs | 2m 41.13s<br>620.6 Ko/s<br>4.735 MB/s<br>1.611 µs |
| **Linear** | 386.90 ms<br>258.5 Mo/s<br>1.926 GB/s<br>3.869 ns | **78.89 ms<br>1268 Mo/s<br>9.444 GB/s<br>0.789 ns** | 7.92 s<br>12.62 Mo/s<br>96.31 MB/s<br>79.22 ns | 7.83 s<br>12.77 Mo/s<br>97.41 MB/s<br>78.33 ns | 7.26 s<br>13.78 Mo/s<br>105.1 MB/s<br>72.58 ns | 5.26 s<br>19.01 Mo/s<br>145.0 MB/s<br>52.60 ns |
| **Random** | 0.44 ms<br>2267 Mo/s<br>16.89 GB/s<br>0.441 ns | **0.42 ms<br>2376 Mo/s<br>17.70 GB/s<br>0.421 ns** | 4.68 s<br>213.8 Ko/s<br>1.631 MB/s<br>4.678 µs | 7.14 s<br>140.0 Ko/s<br>1.068 MB/s<br>7.143 µs | 12.10 s<br>82.63 Ko/s<br>0.630 MB/s<br>12.10 µs | 2.58 s<br>388.2 Ko/s<br>2.961 MB/s<br>2.576 µs |
| **Random Rayon** | 180.01 ms<br>5.555 Mo/s<br>42.38 MB/s<br>180.0 ns | **175.45 ms<br>5.700 Mo/s<br>43.48 MB/s<br>175.5 ns** | 725.79 ms<br>1.378 Mo/s<br>10.51 MB/s<br>725.8 ns | 1.43 s<br>700.7 Ko/s<br>5.346 MB/s<br>1.427 µs | 446.84 ms<br>2.238 Mo/s<br>17.07 MB/s<br>446.8 ns | 189.20 ms<br>5.285 Mo/s<br>40.32 MB/s<br>189.2 ns |
| **Disk Size** | **1.00 GB** | **1.00 GB** | 1.78 GB | 1.00 GB | 3.03 GB | 3.58 GB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
