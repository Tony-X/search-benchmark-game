import org.apache.lucene.analysis.Analyzer;
import org.apache.lucene.analysis.TokenFilter;
import org.apache.lucene.analysis.core.WhitespaceAnalyzer;
import org.apache.lucene.analysis.core.WhitespaceTokenizer;
import org.apache.lucene.analysis.miscellaneous.LengthFilter;
import org.apache.lucene.document.Document;
import org.apache.lucene.document.Field;
import org.apache.lucene.document.LongField;
import org.apache.lucene.document.TextField;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.search.Sort;
import org.apache.lucene.search.SortedNumericSelector;
import org.apache.lucene.store.FSDirectory;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.nio.file.Paths;

import static org.apache.lucene.analysis.standard.StandardTokenizer.MAX_TOKEN_LENGTH_LIMIT;

public class BuildIndex {
    public static void main(String[] args) throws IOException {
        final Path outputPath = Paths.get(args[0]);
        final int index_delete_pct = Integer.valueOf(args[1]);

        final IndexWriterConfig config = new IndexWriterConfig(getTextAnalyzer());
        config.setIndexSort(new Sort(LongField.newSortField("id", false, SortedNumericSelector.Type.MIN)));
        config.setRAMBufferSizeMB(1000);
        int i = 0, num_skipped = 0;
        try (IndexWriter writer = new IndexWriter(FSDirectory.open(outputPath), config)) {
            try (BufferedReader bufferedReader = new BufferedReader(new InputStreamReader(System.in))) {
                final Document document = new Document();

                TextField bodyField = new TextField("body", "", Field.Store.NO);
                LongField idField = new LongField("id", 0);

                document.add(bodyField);
                document.add(idField);

                String line;
                while ((line = bufferedReader.readLine()) != null) {
                    if (line.trim().isEmpty()) {
                        continue;
                    }
                    var parsed_line = line.split("\t");
                    // title date body label
                    if (parsed_line.length != 4) {
                        System.out.println("invalid record, skipping line: " + line);
                        num_skipped += 1;
                        continue;
                    }
                    i += 1;
                    if (i % 100000 == 0) {
                        System.out.println(i);
                    }
                    bodyField.setStringValue(parsed_line[2]);
                    idField.setLongValue(i);
                    writer.addDocument(document);
                }
            }

            writer.commit();
            System.out.println("Merging");
            writer.forceMerge(1, true);

            // Apply deletes
            int num_deleted = 0;
            for (int j = 1; j <= i; j++) {
                if (j % 100 < index_delete_pct) {
                    writer.deleteDocuments(LongField.newExactQuery("id", j));
                    num_deleted++;
                }
            }
            writer.commit();
            System.out.println("Done. Read " + i + " docs." + "Skipped " + num_skipped +
                    " lines. deleted " + num_deleted);


        }
    }

    public static Analyzer getTextAnalyzer() {
        return  new Analyzer() {
            @Override
            protected TokenStreamComponents createComponents(String fieldName) {
                var source = new WhitespaceTokenizer(MAX_TOKEN_LENGTH_LIMIT);
                var filter = new LengthFilter(source, 0, 255);
                return new TokenStreamComponents(source, filter);
            }
        };
    }

}
