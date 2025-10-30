# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## System Information

- **CPU**: Apple M3 Pro
- **CPU Cores**: 12
- **Total Memory**: 36.00 GB
- **OS**: Darwin 26.0.1

## Benchmark 1

**Test**: 10 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 1 pass

**Random Seed**: 42

**Databases**: vecdb, vecdb_old, fjall3, fjall2, redb, lmdb, rocksdb

### Results

| Metric | vecdb | vecdb_old | fjall3 | fjall2 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.11 ms | **0.09 ms** | 1.04 s | 633.38 ms | 5.99 ms | 0.16 ms | 2.53 ms |
| **Write** | 25.51 ms<br>392.1 Mo/s<br>2.921 GB/s<br>2.551 ns | **14.20 ms<br>704.3 Mo/s<br>5.247 GB/s<br>1.420 ns** | 15.37 s<br>650.7 Ko/s<br>4.964 MB/s<br>1.537 µs | 12.88 s<br>776.4 Ko/s<br>5.923 MB/s<br>1.288 µs | 8.08 s<br>1.238 Mo/s<br>9.447 MB/s<br>807.6 ns | 7.21 s<br>1.387 Mo/s<br>10.58 MB/s<br>721.1 ns | 4.47 s<br>2.237 Mo/s<br>17.07 MB/s<br>447.0 ns |
| **len()** | **0.00 ms** | 0.00 ms | 696.50 ms | 750.70 ms | 0.01 ms | 0.00 ms | 1.01 s |
| **Seq** | **6.66 ms<br>1501 Mo/s<br>11.18 GB/s<br>0.666 ns** | 36.16 ms<br>276.6 Mo/s<br>2.060 GB/s<br>3.616 ns | 705.28 ms<br>14.18 Mo/s<br>108.2 MB/s<br>70.53 ns | 766.82 ms<br>13.04 Mo/s<br>99.49 MB/s<br>76.68 ns | 339.94 ms<br>29.42 Mo/s<br>224.4 MB/s<br>33.99 ns | 82.91 ms<br>120.6 Mo/s<br>920.2 MB/s<br>8.291 ns | 1.03 s<br>9.663 Mo/s<br>73.72 MB/s<br>103.5 ns |
| **Seq 4t** | **2.15 ms<br>4655 Mo/s<br>34.68 GB/s<br>0.215 ns** | 247.56 ms<br>40.39 Mo/s<br>308.2 MB/s<br>24.76 ns | 3.25 s<br>3.078 Mo/s<br>23.48 MB/s<br>324.9 ns | 2.16 s<br>4.627 Mo/s<br>35.30 MB/s<br>216.1 ns | 504.35 ms<br>19.83 Mo/s<br>151.3 MB/s<br>50.43 ns | 1.62 s<br>6.165 Mo/s<br>47.04 MB/s<br>162.2 ns | 13.33 s<br>749.9 Ko/s<br>5.721 MB/s<br>1.333 µs |
| **Random** | **0.04 ms<br>2381 Mo/s<br>17.74 GB/s<br>0.420 ns** | 0.05 ms<br>1966 Mo/s<br>14.64 GB/s<br>0.509 ns | 191.59 ms<br>521.9 Ko/s<br>3.982 MB/s<br>1.916 µs | 427.93 ms<br>233.7 Ko/s<br>1.783 MB/s<br>4.279 µs | 116.74 ms<br>856.6 Ko/s<br>6.535 MB/s<br>1.167 µs | 94.23 ms<br>1.061 Mo/s<br>8.097 MB/s<br>942.3 ns | 587.62 ms<br>170.2 Ko/s<br>1.298 MB/s<br>5.876 µs |
| **Random Rayon** | 2.29 ms<br>43.68 Mo/s<br>333.3 MB/s<br>22.89 ns | **2.18 ms<br>45.81 Mo/s<br>349.5 MB/s<br>21.83 ns** | 38.81 ms<br>2.577 Mo/s<br>19.66 MB/s<br>388.1 ns | 170.22 ms<br>587.5 Ko/s<br>4.482 MB/s<br>1.702 µs | 45.31 ms<br>2.207 Mo/s<br>16.84 MB/s<br>453.1 ns | 17.34 ms<br>5.766 Mo/s<br>43.99 MB/s<br>173.4 ns | 154.90 ms<br>645.6 Ko/s<br>4.925 MB/s<br>1.549 µs |
| **Disk Size** | **128.00 MB** | **128.00 MB** | 209.46 MB | 137.11 MB | 514.00 MB | 367.13 MB | 243.85 MB |
---

## Benchmark 2

**Test**: 100 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 1 pass

**Random Seed**: 42

**Databases**: vecdb, vecdb_old, fjall2, fjall3, redb, lmdb

### Results

| Metric | vecdb | vecdb_old | fjall2 | fjall3 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.33 ms | 0.61 ms | 36.92 ms | 264.71 ms | 5.52 ms | **0.17 ms** |
| **Write** | 114.39 ms<br>874.2 Mo/s<br>6.513 GB/s<br>1.144 ns | **110.53 ms<br>904.7 Mo/s<br>6.740 GB/s<br>1.105 ns** | 2m 6.67s<br>789.4 Ko/s<br>6.023 MB/s<br>1.267 µs | 2m 33.23s<br>652.6 Ko/s<br>4.979 MB/s<br>1.532 µs | 1m 24.90s<br>1.178 Mo/s<br>8.987 MB/s<br>849.0 ns | 2m 45.40s<br>604.6 Ko/s<br>4.613 MB/s<br>1.654 µs |
| **len()** | **0.00 ms** | 0.00 ms | 8.23 s | 8.31 s | 0.13 ms | 0.00 ms |
| **Seq** | **67.22 ms<br>1488 Mo/s<br>11.08 GB/s<br>0.672 ns** | 385.73 ms<br>259.2 Mo/s<br>1.932 GB/s<br>3.857 ns | 8.69 s<br>11.51 Mo/s<br>87.81 MB/s<br>86.88 ns | 8.44 s<br>11.85 Mo/s<br>90.43 MB/s<br>84.37 ns | 5.70 s<br>17.55 Mo/s<br>133.9 MB/s<br>56.99 ns | 8.04 s<br>12.45 Mo/s<br>94.95 MB/s<br>80.35 ns |
| **Seq 4t** | **87.49 ms<br>1143 Mo/s<br>8.516 GB/s<br>0.875 ns** | 2.21 s<br>45.29 Mo/s<br>345.5 MB/s<br>22.08 ns | 24.44 s<br>4.092 Mo/s<br>31.22 MB/s<br>244.4 ns | 33.38 s<br>2.996 Mo/s<br>22.86 MB/s<br>333.8 ns | 9.20 s<br>10.87 Mo/s<br>82.95 MB/s<br>91.97 ns | 25.98 s<br>3.849 Mo/s<br>29.37 MB/s<br>259.8 ns |
| **Random** | 0.42 ms<br>2407 Mo/s<br>17.94 GB/s<br>0.415 ns | **0.41 ms<br>2412 Mo/s<br>17.97 GB/s<br>0.415 ns** | 6.98 s<br>143.2 Ko/s<br>1.093 MB/s<br>6.982 µs | 3.99 s<br>250.7 Ko/s<br>1.913 MB/s<br>3.989 µs | 12.61 s<br>79.33 Ko/s<br>0.605 MB/s<br>12.61 µs | 1.73 s<br>577.0 Ko/s<br>4.402 MB/s<br>1.733 µs |
| **Random Rayon** | 25.55 ms<br>39.14 Mo/s<br>298.6 MB/s<br>25.55 ns | **21.63 ms<br>46.23 Mo/s<br>352.7 MB/s<br>21.63 ns** | 1.38 s<br>727.0 Ko/s<br>5.547 MB/s<br>1.375 µs | 615.52 ms<br>1.625 Mo/s<br>12.40 MB/s<br>615.5 ns | 411.75 ms<br>2.429 Mo/s<br>18.53 MB/s<br>411.8 ns | 197.42 ms<br>5.065 Mo/s<br>38.64 MB/s<br>197.4 ns |
| **Disk Size** | **1.00 GB** | **1.00 GB** | 1.00 GB | 1.78 GB | 3.03 GB | 3.58 GB |
---

## Benchmark 3

**Test**: 200 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 1 pass

**Random Seed**: 42

**Databases**: vecdb, vecdb_old, fjall2, fjall3, redb, lmdb

### Results

| Metric | vecdb | vecdb_old | fjall2 | fjall3 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 1.20 ms | 0.65 ms | 93.15 ms | 531.50 ms | 8.11 ms | **0.16 ms** |
| **Write** | **252.18 ms<br>793.1 Mo/s<br>5.909 GB/s<br>1.261 ns** | 253.21 ms<br>789.9 Mo/s<br>5.885 GB/s<br>1.266 ns | 4m 32.17s<br>734.8 Ko/s<br>5.606 MB/s<br>1.361 µs | 5m 31.58s<br>603.2 Ko/s<br>4.602 MB/s<br>1.658 µs | 3m 3.21s<br>1.092 Mo/s<br>8.328 MB/s<br>916.1 ns | 10m 15.93s<br>324.7 Ko/s<br>2.477 MB/s<br>3.080 µs |
| **len()** | **0.00 ms** | 0.00 ms | 16.49 s | 16.61 s | 0.17 ms | 0.00 ms |
| **Seq** | **241.23 ms<br>829.1 Mo/s<br>6.177 GB/s<br>1.206 ns** | 2.72 s<br>73.65 Mo/s<br>561.9 MB/s<br>13.58 ns | 16.65 s<br>12.01 Mo/s<br>91.63 MB/s<br>83.26 ns | 16.82 s<br>11.89 Mo/s<br>90.70 MB/s<br>84.12 ns | 30.46 s<br>6.565 Mo/s<br>50.09 MB/s<br>152.3 ns | 46.87 s<br>4.267 Mo/s<br>32.55 MB/s<br>234.4 ns |
| **Seq 4t** | **274.78 ms<br>727.9 Mo/s<br>5.423 GB/s<br>1.374 ns** | 3.97 s<br>50.35 Mo/s<br>384.1 MB/s<br>19.86 ns | 48.94 s<br>4.086 Mo/s<br>31.18 MB/s<br>244.7 ns | 1m 9.23s<br>2.889 Mo/s<br>22.04 MB/s<br>346.2 ns | 32.31 s<br>6.191 Mo/s<br>47.23 MB/s<br>161.5 ns | 1m 7.28s<br>2.973 Mo/s<br>22.68 MB/s<br>336.4 ns |
| **Random** | 1.64 ms<br>1219 Mo/s<br>9.082 GB/s<br>0.820 ns | **0.83 ms<br>2423 Mo/s<br>18.05 GB/s<br>0.413 ns** | 31.95 s<br>62.59 Ko/s<br>0.478 MB/s<br>15.98 µs | 57.84 s<br>34.58 Ko/s<br>0.264 MB/s<br>28.92 µs | 4m 34.70s<br>7.281 Ko/s<br>0.056 MB/s<br>137.4 µs | 19.85 s<br>100.8 Ko/s<br>0.769 MB/s<br>9.925 µs |
| **Random Rayon** | **817.37 ms<br>2.447 Mo/s<br>18.67 MB/s<br>408.7 ns** | 846.32 ms<br>2.363 Mo/s<br>18.03 MB/s<br>423.2 ns | 3.94 s<br>508.0 Ko/s<br>3.876 MB/s<br>1.968 µs | 5.67 s<br>352.5 Ko/s<br>2.689 MB/s<br>2.837 µs | 943.86 ms<br>2.119 Mo/s<br>16.17 MB/s<br>471.9 ns | 1.15 s<br>1.733 Mo/s<br>13.23 MB/s<br>576.9 ns |
| **Disk Size** | 2.00 GB | 2.00 GB | **2.00 GB** | 3.57 GB | 6.06 GB | 6.99 GB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
