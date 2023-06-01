DEV_CORPUS := $(shell pwd)/corpus/20k-enwiki-20120502-lines-1k-fixed-utf8-with-random-label.txt
#REAL_CORPUS := "$(shell pwd)/corpus/enwiki-20120502-lines-1k-fixed-utf8-with-random-label.txt"
REAL_CORPUS := "/lucenedata/enwiki/enwiki-20120502-lines-1k-fixed-utf8-with-random-label.txt"

WIKI_SRC = "http://home.apache.org/~mikemccand/enwiki-20120502-lines-1k-fixed-utf8-with-random-label.txt.lzma"

export

# What to bench?
# COMMANDS ?= TOP_10
COMMANDS ?= TOP_10_COUNT COUNT
ENGINES ?= tantivy-0.19 lucene-9.5.0
QUERY_FILE ?= queries/basic_queries.jsonl

# Index settings
INDEX_DELETE_PCT ?= 2

# Benchmark client settings
WARMUP_ITER ?= 1
NUM_ITER ?= 10

# Serving options
PORT ?= 12345

help:
	@grep '^[^#[:space:]].*:' Makefile

all: index

corpus:
	@echo "--- Downloading $(WIKI_SRC) ---"
	@curl -s $(WIKI_SRC) | lzma -d - > $(CORPUS)

clean:
	@echo "--- Cleaning directories ---"
	@rm -fr results
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make clean ; done

clean-index:
	@echo "--- Cleaning indices only ---"
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make clean-index ; done


index:
	@echo "--- Indexing real corpus ---"
	@export CORPUS=$(REAL_CORPUS)
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make CORPUS=$(REAL_CORPUS) index ; done

dev-index:
	@echo "--- Indexing dev corpus ---"
	@echo "$(DEV_CORPUS)"
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make CORPUS=$(DEV_CORPUS) index ; done

bench:
	@echo "--- Benchmarking ---"
	@rm -fr results
	@mkdir results
	@python3 src/client.py $(QUERY_FILE) $(ENGINES)

compile:
	@echo "--- Compiling binaries ---"
	@for engine in $(ENGINES); do cd ${shell pwd}/engines/$$engine && make compile ; done

serve:
	@echo "--- Serving results ---"
	@cp results.json web/build/results.json
	@cd web/build && python3 -m http.server $(PORT) --bind 0.0.0.0
