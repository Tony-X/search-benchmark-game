export CORPUS

HEAP = 4g

NAME ?= "Lucene 9.8.0"
clean:
	@echo "--- Cleaning $(NAME) ---"
	@rm -fr idx
	@rm -fr build

clean-index:
	@rm -fr idx

compile: build

serve:
	@java -Xmx$(HEAP) -Xms$(HEAP) -XX:+UseParallelGC -Dorg.apache.lucene.store.MMapDirectory.enableMemorySegments=false -cp build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar DoQuery idx

index: idx

build: src/main/java/*.java
	@echo "--- Building $(NAME) ---"
	@gradle clean shadowJar

idx: build
	@echo "--- Indexing $(NAME) with %$(INDEX_DELETE_PCT) deletes ---"
	java -server -Dorg.apache.lucene.store.MMapDirectory.enableMemorySegments=false -cp build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar BuildIndex idx $(INDEX_DELETE_PCT) < $(CORPUS)
