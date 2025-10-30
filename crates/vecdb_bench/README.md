# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## System Information

- **CPU**: Apple M3 Pro
- **CPU Cores**: 12
- **Total Memory**: 36.00 GB
- **OS**: Darwin 26.0.1

## Benchmark 1

**Test**: 10 million sequential u64 writes, linear reads, and 1 million random reads.

**Iterations**: 1 pass

**Random Seed**: 42

**Databases**: vecdb, vecdb_old, fjall2, fjall3, redb, lmdb, rocksdb

### Results

| Metric | vecdb | vecdb_old | fjall2 | fjall3 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.10 ms | **0.09 ms** | 649.56 ms | 1.05 s | 12.00 ms | 0.14 ms | 2.60 ms |
| **Write** | 14.06 ms<br>711.2 Mo/s<br>5.299 GB/s<br>1.406 ns | **13.70 ms<br>729.7 Mo/s<br>5.436 GB/s<br>1.370 ns** | 8.96 s<br>1.117 Mo/s<br>8.519 MB/s<br>895.6 ns | 8.39 s<br>1.192 Mo/s<br>9.095 MB/s<br>838.8 ns | 8.31 s<br>1.203 Mo/s<br>9.180 MB/s<br>831.1 ns | 7.07 s<br>1.415 Mo/s<br>10.80 MB/s<br>706.6 ns | 4.38 s<br>2.285 Mo/s<br>17.43 MB/s<br>437.6 ns |
| **len()** | 0.00 ms | **0.00 ms** | 684.03 ms | 696.51 ms | 0.01 ms | 0.00 ms | 1.00 s |
| **Linear Read** | **6.50 ms<br>1538 Mo/s<br>11.46 GB/s<br>0.650 ns** | 42.61 ms<br>234.7 Mo/s<br>1.749 GB/s<br>4.261 ns | 705.46 ms<br>14.18 Mo/s<br>108.1 MB/s<br>70.55 ns | 697.67 ms<br>14.33 Mo/s<br>109.4 MB/s<br>69.77 ns | 334.53 ms<br>29.89 Mo/s<br>228.1 MB/s<br>33.45 ns | 81.91 ms<br>122.1 Mo/s<br>931.4 MB/s<br>8.191 ns | 1.00 s<br>9.963 Mo/s<br>76.01 MB/s<br>100.4 ns |
| **Seq 2t** | **3.54 ms<br>2827 Mo/s<br>21.06 GB/s<br>0.354 ns** | 161.96 ms<br>61.74 Mo/s<br>471.1 MB/s<br>16.20 ns | 2.90 s<br>3.452 Mo/s<br>26.34 MB/s<br>289.7 ns | 5.13 s<br>1.950 Mo/s<br>14.88 MB/s<br>512.7 ns | 699.68 ms<br>14.29 Mo/s<br>109.0 MB/s<br>69.97 ns | 3.33 s<br>3.007 Mo/s<br>22.94 MB/s<br>332.6 ns | 21.49 s<br>465.3 Ko/s<br>3.550 MB/s<br>2.149 µs |
| **Seq 4t** | **2.02 ms<br>4959 Mo/s<br>36.95 GB/s<br>0.202 ns** | 243.90 ms<br>41.00 Mo/s<br>312.8 MB/s<br>24.39 ns | 2.09 s<br>4.795 Mo/s<br>36.58 MB/s<br>208.5 ns | 3.21 s<br>3.111 Mo/s<br>23.73 MB/s<br>321.5 ns | 502.04 ms<br>19.92 Mo/s<br>152.0 MB/s<br>50.20 ns | 1.56 s<br>6.419 Mo/s<br>48.97 MB/s<br>155.8 ns | 12.57 s<br>795.6 Ko/s<br>6.070 MB/s<br>1.257 µs |
| **Seq 8t** | **1.83 ms<br>5464 Mo/s<br>40.71 GB/s<br>0.183 ns** | 1.01 s<br>9.875 Mo/s<br>75.34 MB/s<br>101.3 ns | 3.51 s<br>2.851 Mo/s<br>21.75 MB/s<br>350.8 ns | 2.69 s<br>3.720 Mo/s<br>28.38 MB/s<br>268.8 ns | 960.05 ms<br>10.42 Mo/s<br>79.47 MB/s<br>96.01 ns | 993.38 ms<br>10.07 Mo/s<br>76.80 MB/s<br>99.34 ns | 7.35 s<br>1.360 Mo/s<br>10.38 MB/s<br>735.0 ns |
| **Random 1t** | 442.14 ms<br>2.262 Mo/s<br>17.26 MB/s<br>442.1 ns | **0.43 ms<br>2316 Mo/s<br>17.25 GB/s<br>0.432 ns** | 4.45 s<br>224.9 Ko/s<br>1.716 MB/s<br>4.447 µs | 1.55 s<br>645.8 Ko/s<br>4.927 MB/s<br>1.549 µs | 579.86 ms<br>1.725 Mo/s<br>13.16 MB/s<br>579.9 ns | 823.42 ms<br>1.214 Mo/s<br>9.266 MB/s<br>823.4 ns | 5.74 s<br>174.2 Ko/s<br>1.329 MB/s<br>5.739 µs |
| **Random 4t** | 358.96 ms<br>2.786 Mo/s<br>21.25 MB/s<br>359.0 ns | **2.68 ms<br>373.7 Mo/s<br>2.784 GB/s<br>2.676 ns** | 1.34 s<br>743.7 Ko/s<br>5.674 MB/s<br>1.345 µs | 463.41 ms<br>2.158 Mo/s<br>16.46 MB/s<br>463.4 ns | 185.51 ms<br>5.391 Mo/s<br>41.13 MB/s<br>185.5 ns | 219.55 ms<br>4.555 Mo/s<br>34.75 MB/s<br>219.5 ns | 1.88 s<br>532.9 Ko/s<br>4.066 MB/s<br>1.877 µs |
| **Random 8t** | 738.09 ms<br>1.355 Mo/s<br>10.34 MB/s<br>738.1 ns | **3.23 ms<br>309.6 Mo/s<br>2.306 GB/s<br>3.230 ns** | 1.25 s<br>798.0 Ko/s<br>6.089 MB/s<br>1.253 µs | 352.61 ms<br>2.836 Mo/s<br>21.64 MB/s<br>352.6 ns | 171.24 ms<br>5.840 Mo/s<br>44.56 MB/s<br>171.2 ns | 158.12 ms<br>6.324 Mo/s<br>48.25 MB/s<br>158.1 ns | 1.80 s<br>557.1 Ko/s<br>4.250 MB/s<br>1.795 µs |
| **Random 12t** | 815.01 ms<br>1.227 Mo/s<br>9.361 MB/s<br>815.0 ns | **3.71 ms<br>269.4 Mo/s<br>2.007 GB/s<br>3.712 ns** | 1.76 s<br>568.4 Ko/s<br>4.337 MB/s<br>1.759 µs | 324.58 ms<br>3.081 Mo/s<br>23.51 MB/s<br>324.6 ns | 165.96 ms<br>6.026 Mo/s<br>45.97 MB/s<br>166.0 ns | 116.32 ms<br>8.597 Mo/s<br>65.59 MB/s<br>116.3 ns | 1.57 s<br>636.2 Ko/s<br>4.854 MB/s<br>1.572 µs |
| **Random 16t** | 815.36 ms<br>1.226 Mo/s<br>9.357 MB/s<br>815.4 ns | **3.66 ms<br>273.4 Mo/s<br>2.037 GB/s<br>3.657 ns** | 2.11 s<br>474.1 Ko/s<br>3.617 MB/s<br>2.109 µs | 271.78 ms<br>3.679 Mo/s<br>28.07 MB/s<br>271.8 ns | 148.30 ms<br>6.743 Mo/s<br>51.45 MB/s<br>148.3 ns | 104.91 ms<br>9.532 Mo/s<br>72.72 MB/s<br>104.9 ns | 1.52 s<br>655.9 Ko/s<br>5.004 MB/s<br>1.525 µs |
| **Random Rayon** | 821.20 ms<br>1.218 Mo/s<br>9.291 MB/s<br>821.2 ns | **3.37 ms<br>296.5 Mo/s<br>2.209 GB/s<br>3.372 ns** | 1.59 s<br>629.6 Ko/s<br>4.804 MB/s<br>1.588 µs | 273.52 ms<br>3.656 Mo/s<br>27.89 MB/s<br>273.5 ns | 136.61 ms<br>7.320 Mo/s<br>55.85 MB/s<br>136.6 ns | 104.53 ms<br>9.566 Mo/s<br>72.99 MB/s<br>104.5 ns | 1.54 s<br>649.5 Ko/s<br>4.955 MB/s<br>1.540 µs |
| **Disk Size** | **128.00 MB** | **128.00 MB** | 137.11 MB | 209.45 MB | 514.00 MB | 367.13 MB | 243.98 MB |
---

## Benchmark 2

**Test**: 100 million sequential u64 writes, linear reads, and 10 million random reads.

**Iterations**: 1 pass

**Random Seed**: 42

**Databases**: vecdb, vecdb_old, fjall2, fjall3, redb, lmdb

### Results

| Metric | vecdb | vecdb_old | fjall2 | fjall3 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.15 ms | **0.10 ms** | 40.56 ms | 243.36 ms | 5.17 ms | 0.20 ms |
| **Write** | 112.37 ms<br>889.9 Mo/s<br>6.631 GB/s<br>1.124 ns | **94.54 ms<br>1058 Mo/s<br>7.881 GB/s<br>0.945 ns** | 1m 31.62s<br>1.091 Mo/s<br>8.327 MB/s<br>916.2 ns | 1m 25.36s<br>1.171 Mo/s<br>8.938 MB/s<br>853.6 ns | 1m 28.01s<br>1.136 Mo/s<br>8.668 MB/s<br>880.1 ns | 2m 38.40s<br>631.3 Ko/s<br>4.817 MB/s<br>1.584 µs |
| **len()** | 0.00 ms | **0.00 ms** | 7.12 s | 7.64 s | 0.01 ms | 0.00 ms |
| **Linear Read** | **62.78 ms<br>1593 Mo/s<br>11.87 GB/s<br>0.628 ns** | 423.27 ms<br>236.3 Mo/s<br>1.760 GB/s<br>4.233 ns | 7.28 s<br>13.74 Mo/s<br>104.8 MB/s<br>72.79 ns | 7.75 s<br>12.91 Mo/s<br>98.47 MB/s<br>77.48 ns | 3.27 s<br>30.57 Mo/s<br>233.2 MB/s<br>32.71 ns | 940.08 ms<br>106.4 Mo/s<br>811.6 MB/s<br>9.401 ns |
| **Seq 2t** | **33.94 ms<br>2946 Mo/s<br>21.95 GB/s<br>0.339 ns** | 1.13 s<br>88.24 Mo/s<br>673.2 MB/s<br>11.33 ns | 31.56 s<br>3.168 Mo/s<br>24.17 MB/s<br>315.6 ns | 52.20 s<br>1.916 Mo/s<br>14.61 MB/s<br>522.0 ns | 10.98 s<br>9.106 Mo/s<br>69.48 MB/s<br>109.8 ns | 49.86 s<br>2.006 Mo/s<br>15.30 MB/s<br>498.6 ns |
| **Seq 4t** | **21.13 ms<br>4734 Mo/s<br>35.27 GB/s<br>0.211 ns** | 2.15 s<br>46.48 Mo/s<br>354.6 MB/s<br>21.52 ns | 22.72 s<br>4.402 Mo/s<br>33.58 MB/s<br>227.2 ns | 31.47 s<br>3.177 Mo/s<br>24.24 MB/s<br>314.7 ns | 8.83 s<br>11.33 Mo/s<br>86.41 MB/s<br>88.29 ns | 25.83 s<br>3.871 Mo/s<br>29.54 MB/s<br>258.3 ns |
| **Seq 8t** | **13.65 ms<br>7323 Mo/s<br>54.56 GB/s<br>0.137 ns** | 10.27 s<br>9.734 Mo/s<br>74.27 MB/s<br>102.7 ns | 37.58 s<br>2.661 Mo/s<br>20.30 MB/s<br>375.8 ns | 29.50 s<br>3.389 Mo/s<br>25.86 MB/s<br>295.0 ns | 16.08 s<br>6.219 Mo/s<br>47.45 MB/s<br>160.8 ns | 14.98 s<br>6.676 Mo/s<br>50.93 MB/s<br>149.8 ns |
| **Random 1t** | 4.80 s<br>2.085 Mo/s<br>15.91 MB/s<br>479.6 ns | **4.10 ms<br>2438 Mo/s<br>18.16 GB/s<br>0.410 ns** | 59.82 s<br>167.2 Ko/s<br>1.275 MB/s<br>5.982 µs | 26.39 s<br>378.9 Ko/s<br>2.891 MB/s<br>2.639 µs | 18.54 s<br>539.5 Ko/s<br>4.116 MB/s<br>1.854 µs | 10.26 s<br>974.9 Ko/s<br>7.438 MB/s<br>1.026 µs |
| **Random 4t** | 3.74 s<br>2.674 Mo/s<br>20.40 MB/s<br>374.0 ns | **28.22 ms<br>354.3 Mo/s<br>2.640 GB/s<br>2.822 ns** | 17.41 s<br>574.4 Ko/s<br>4.383 MB/s<br>1.741 µs | 8.05 s<br>1.242 Mo/s<br>9.472 MB/s<br>805.5 ns | 7.11 s<br>1.407 Mo/s<br>10.73 MB/s<br>710.8 ns | 2.65 s<br>3.772 Mo/s<br>28.78 MB/s<br>265.1 ns |
| **Random 8t** | 6.82 s<br>1.466 Mo/s<br>11.19 MB/s<br>681.9 ns | **32.13 ms<br>311.2 Mo/s<br>2.319 GB/s<br>3.213 ns** | 12.78 s<br>782.7 Ko/s<br>5.972 MB/s<br>1.278 µs | 6.84 s<br>1.461 Mo/s<br>11.15 MB/s<br>684.4 ns | 44.29 s<br>225.8 Ko/s<br>1.723 MB/s<br>4.429 µs | 4.68 s<br>2.138 Mo/s<br>16.32 MB/s<br>467.6 ns |
| **Random 12t** | 8.35 s<br>1.198 Mo/s<br>9.138 MB/s<br>834.9 ns | **421.43 ms<br>23.73 Mo/s<br>181.0 MB/s<br>42.14 ns** | 13.22 s<br>756.5 Ko/s<br>5.771 MB/s<br>1.322 µs | 4.63 s<br>2.158 Mo/s<br>16.46 MB/s<br>463.5 ns | 6.04 s<br>1.656 Mo/s<br>12.63 MB/s<br>603.9 ns | 1.35 s<br>7.395 Mo/s<br>56.42 MB/s<br>135.2 ns |
| **Random 16t** | 8.45 s<br>1.184 Mo/s<br>9.033 MB/s<br>844.6 ns | **349.94 ms<br>28.58 Mo/s<br>218.0 MB/s<br>34.99 ns** | 13.27 s<br>753.7 Ko/s<br>5.750 MB/s<br>1.327 µs | 4.84 s<br>2.065 Mo/s<br>15.76 MB/s<br>484.2 ns | 6.83 s<br>1.464 Mo/s<br>11.17 MB/s<br>683.2 ns | 1.73 s<br>5.784 Mo/s<br>44.13 MB/s<br>172.9 ns |
| **Random Rayon** | 67.12 ms<br>149.0 Mo/s<br>1.110 GB/s<br>6.712 ns | **56.15 ms<br>178.1 Mo/s<br>1.327 GB/s<br>5.615 ns** | 13.20 s<br>757.8 Ko/s<br>5.781 MB/s<br>1.320 µs | 4.71 s<br>2.125 Mo/s<br>16.22 MB/s<br>470.5 ns | 4.46 s<br>2.244 Mo/s<br>17.12 MB/s<br>445.7 ns | 1.33 s<br>7.530 Mo/s<br>57.45 MB/s<br>132.8 ns |
| **Disk Size** | **1.00 GB** | **1.00 GB** | 1.00 GB | 1.78 GB | 3.03 GB | 3.58 GB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
