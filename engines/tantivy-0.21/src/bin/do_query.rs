use tantivy::collector::{Count, TopDocs};
use tantivy::query::{QueryParser, Weight};
use tantivy::{DocId, Index, Score, SegmentReader, TERMINATED};
use tantivy_bench::get_tokenizer_manager;

use std::collections::BinaryHeap;
use std::env;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner(Path::new(&args[1])).unwrap()
}

struct Float(Score);

use std::cmp::Ordering;

impl Eq for Float {}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

fn checkpoints_pruning(
    weight: &dyn Weight,
    reader: &SegmentReader,
    n: usize,
) -> tantivy::Result<Vec<(DocId, Score, Score)>> {
    let mut heap: BinaryHeap<Float> = BinaryHeap::with_capacity(n);
    let mut checkpoints: Vec<(DocId, Score, Score)> = Vec::new();
    let mut limit: Score = 0.0;
    weight.for_each_pruning(Score::MIN, reader, &mut |doc, score| {
        checkpoints.push((doc, score, score));
        // println!("pruning doc={} score={} limit={}", doc, score, limit);
        heap.push(Float(score));
        if heap.len() > n {
            heap.pop().unwrap();
        }
        limit = heap.peek().unwrap().0;
        limit
    })?;
    Ok(checkpoints)
}

fn checkpoints_no_pruning(
    weight: &dyn Weight,
    reader: &SegmentReader,
    n: usize,
) -> tantivy::Result<Vec<(DocId, Score, Score)>> {
    let mut heap: BinaryHeap<Float> = BinaryHeap::with_capacity(n);
    let mut checkpoints: Vec<(DocId, Score, Score)> = Vec::new();
    let mut scorer = weight.scorer(reader, 1.0)?;
    let mut limit = Score::MIN;
    loop {
        if scorer.doc() == TERMINATED {
            break;
        }
        let doc = scorer.doc();
        let score = scorer.score();
        if score > limit {
            // println!("nopruning doc={} score={} limit={}", doc, score, limit);
            checkpoints.push((doc, limit, score));
            heap.push(Float(score));
            if heap.len() > n {
                heap.pop().unwrap();
            }
            limit = heap.peek().unwrap().0;
        }
        scorer.advance();
    }
    Ok(checkpoints)
}

fn _assert_nearly_equals(left: Score, right: Score) -> bool {
    (left - right).abs() * 2.0 / (left + right).abs() < 0.000001
}

fn main_inner(index_dir: &Path) -> tantivy::Result<()> {
    let index = Index::open_in_dir(index_dir).expect("failed to open index");
    let text_field = index.schema().get_field("body").expect("no all field?!");
    let query_parser = QueryParser::new(index.schema(), vec![text_field], get_tokenizer_manager());
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let fields: Vec<&str> = line.split('\t').collect();

        let command = fields[0];
        if command != "TOP_N_DOCS" {
            assert_eq!(
                fields.len(),
                2,
                "Expected a line in the format <COMMAND> query."
            );
        }
        let query = query_parser.parse_query(fields[1])?;
        let t0 = Instant::now();
        let result: String = match command {
            "COUNT" => query.count(&searcher)?.to_string(),
            "TOP_10" => {
                top_n_total_hits(10, &searcher, &query)
            }
            "TOP_100" => {
                top_n_total_hits(100, &searcher, &query)
            }
            "TOP_10_COUNT" => {
                let (_top_docs, count) =
                    searcher.search(&query, &(TopDocs::with_limit(10), Count))?;
                count.to_string()
            }
            "TOP_N_DOCS" => {
                assert_eq!(
                    fields.len(),
                    3,
                    "Expect TOP_N_DOCS command to take <QUERY> <TOP_N>"
                );
                let n: usize = fields[2].parse().unwrap();
                let (top_docs, _count) =
                    searcher.search(&query, &(TopDocs::with_limit(n), Count))?;
                let doc_ids: Vec<String> = top_docs
                    .into_iter()
                    .map(|x| x.1.doc_id.to_string())
                    .collect();
                doc_ids.len().to_string() + " " + &doc_ids.join(" ").to_string()
            }
            "DEBUG_TOP_10" => {
                let weight = query.weight(tantivy::query::EnableScoring::enabled_from_searcher(&searcher))?;
                for reader in searcher.segment_readers() {
                    let _checkpoints_left = checkpoints_no_pruning(&*weight, reader, 10)?;
                    let _checkpoints_right = checkpoints_pruning(&*weight, reader, 10)?;
                }
                // TODO: this is weird
                "0".to_string()
            }
            _ => {
                // TODO: this is weird
                "UNSUPPORTED".to_string()
            }
        };

        let t1 = Instant::now();
        println!("{} {}", (t1 - t0).as_nanos(), result);
        // TODO: is this correct???
        // #14: paranoia
        stdout.flush()?;
    }

    Ok(())
}

fn top_n_total_hits(limit: usize, searcher: &tantivy::Searcher, query: &dyn tantivy::query::Query) -> String {
    let _top_docs = searcher.search(query, &TopDocs::with_limit(limit)).unwrap();
    _top_docs.len().to_string()
}
