
# What is this repo about?
This repository is forked from https://github.com/quickwit-oss/search-benchmark-game.

It aims to achieve a close comparison between Tantivy and Lucene using the same search workload from [luceneutil](https://github.com/mikemccand/luceneutil).

The latest results can be found [here](https://tony-x.github.io/search-benchmark-game/).

Want to run the benchmark or make changes? here is the [development guide](#development-guide) 

# The benchmark
## Workload
The corpus used in the benchmark is a snapshot of the English Wikipedia text.

```
# wget http://home.apache.org/~mikemccand/enwiki-20120502-lines-1k-fixed-utf8-with-random-label.txt.lzma
```

The search tasks are from [here](https://github.com/mikemccand/luceneutil/blob/master/tasks/wikimedium.1M.nostopwords.tasks). This repository has a copy, too.

As of now, this benchmark uses only basic text queries which includes:
```
TermQuery
BooleanQuery
PhraseQuery (with slop)
```

## Engine details
### Common
* Text analysis: Both engine use a simple whitespace tokenizer.
* Both indices are force merged down to a single segment
* A single search thread executes each task TK times, discards the first TK (warmup), and records the minimum time of the remaining 8.
* The search thread is a simple Rust client, spwaning a sub-process running either Tantivy or Lucene and communicating over a local Unix pipe

### Tantivy
* Version: 0.19
* Rust version: ???

Features: All default

### Lucene
* Version: 9.5.0
* JDK version/flags: ???
* Disabled query cache.


## Methodology
The benchmark uses a client that simulates a closed-loop system, where a new query is sent only after the completion of the previous one. This is to measure the lowest latency from each engine.

The workload is run against both engines in multiple iterations, including a warmup run at the beginning.

# Development Guide
## Tooling
* `make` is used to manage tasks.
* You need rust. The recommended way to install it via [rustup](https://www.rust-lang.org/tools/install)
* JDK 17+. The are plenty of ways to install it.
* Gardle - 8.1 or up.

## Quick verification
```
make clean

# build the engines, also make indices using the dev corpus
# the dev corpus is a down-sampled version of the larger corpus
# this is useful for fast development/iteration
make dev-index  

# run the benchmark
make bench

# serve the results
make serve
```

## If that all works, you can build the complete index (takes longer, but more accurate results):


```
# This downloads the 33M documents of English wikipedia text (NOTE: ~31 GB, stored locally under `corpus/*` by default
make corpus

make index
make bench
make serve

# Note: make can take a one liner like this
make index bench serve
```

## About the results...
This repo is still work-in-progress of building the trust of the results. It tries to make an apples-to-apples comparison as much as possible. 

Results will vary depending on your hardware!
