use futures::executor::block_on;
use std::env;
use std::io::BufRead;
use std::path::Path;
use tantivy::schema::{NumericOptions, Schema, TEXT};
use tantivy::{doc, IndexBuilder, IndexSettings, IndexSortByField, Order, Term};
use tantivy_bench::get_tokenizer_manager;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(Path::new(&args[1]), i32::from_str_radix(&args[2], 10).unwrap()).unwrap();
}

fn main_inner(output_dir: &Path, index_delete_pct: i32) -> tantivy::Result<()> {
    println!("Build index at `{}` with delete_pct {}%", output_dir.display(), index_delete_pct);

    let mut schema_builder = Schema::builder();

    let body = schema_builder.add_text_field(
        "body",
        TEXT.set_indexing_options(
            TEXT.get_indexing_options()
                .unwrap()
                .clone()
                .set_tokenizer("whitespace"),
        ),
    );
    let id_field = schema_builder.add_u64_field(
        "id",
        NumericOptions::default()
            .set_indexed()
            .set_fast(),
    );
    let schema = schema_builder.build();

    let index = IndexBuilder::new()
        .schema(schema)
        .tokenizers(get_tokenizer_manager())
        .settings(IndexSettings {
            sort_by_field: Some(IndexSortByField {
                order: Order::Asc,
                field: "id".into(),
            }),
	    docstore_compress_dedicated_thread: false,
            ..IndexSettings::default()
        })
        .create_in_dir(output_dir)
        .expect("Failed to create index");

    let mut i = 0;
    let mut num_skipped = 0;
    {
        let mut index_writer = index
            .writer_with_num_threads(4, 2_000_000_000)
            .expect("failed to create index writer");
        let stdin = std::io::stdin();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            // (title, date, body, label)
            let parsed_line: Vec<&str> = line.split('\t').collect();
            if parsed_line.len() != 4 {
                println!("Skipping malformed line: {}", line);
                num_skipped += 1;
                continue;
            }
            i += 1;
            if i % 100_000 == 0 {
                println!("{}", i);
            }
            let doc = doc!(
                id_field => i as u64,
                body => parsed_line[2]
            );
            index_writer.add_document(doc).unwrap();
        }

        index_writer.commit()?;
        index_writer.wait_merging_threads()?;
    }
    let segment_ids = index.searchable_segment_ids()?;
    let mut index_writer = index
        .writer(1_500_000_000)
        .expect("failed to create index writer");
    block_on(index_writer.merge(&segment_ids))?;

    // Apply deletes
    let total_indexed = i;
    let mut num_deleted = 0;
    for i in 1..=total_indexed {
        if i % 100 < index_delete_pct {
            index_writer.delete_term(Term::from_field_u64(id_field, i as u64));
            num_deleted += 1;
        }
    }
    index_writer.commit()?;

    block_on(index_writer.garbage_collect_files())?;
    println!("Done. Read {i} docs, skipped {num_skipped}, deleted {num_deleted}");
    Ok(())
}
