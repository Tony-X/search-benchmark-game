NAME := "tantivy 0.19"
export CORPUS

clean:
	@rm -fr idx
	@rm -fr target

clean-index:
	@rm -fr idx

index: target/release/build_index
	@echo "--- Indexing $(NAME) with $(INDEX_DELETE_PCT)% deletes ---"
	@mkdir -p idx
	@target/release/build_index idx $(INDEX_DELETE_PCT) < $(CORPUS)

serve: target/release/do_query
	@target/release/do_query idx

target/release/%: src/bin/%.rs src/lib.rs
	@echo "--- Building tantivy's binary ---"
	@cargo build --release --bin $(notdir $@)
