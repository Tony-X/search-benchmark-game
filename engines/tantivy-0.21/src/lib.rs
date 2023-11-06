use tantivy::tokenizer::{RemoveLongFilter, TextAnalyzer, TokenizerManager};
use whitespace_tokenizer_fork::WhitespaceTokenizer;

pub fn get_tokenizer_manager() -> TokenizerManager {
    let tokenzier_manager = TokenizerManager::default();
    let tokenizer = TextAnalyzer::builder(WhitespaceTokenizer).filter(RemoveLongFilter::limit(256)).build();
    tokenzier_manager.register("whitespace", tokenizer);
    tokenzier_manager
}

mod whitespace_tokenizer_fork {

    use tantivy::tokenizer::{Token, Tokenizer, TokenStream};
    use std::str::CharIndices;

    /// Tokenize the text by splitting on whitespaces.
    #[derive(Clone)]
    pub struct WhitespaceTokenizer;

    pub struct WhitespaceTokenStream<'a> {
        text: &'a str,
        chars: CharIndices<'a>,
        token: Token,
    }

    impl Tokenizer for WhitespaceTokenizer {
        type TokenStream<'a> = WhitespaceTokenStream<'a>;

        fn token_stream<'a>(&'a mut self, text: &'a str) -> WhitespaceTokenStream<'a> {
            WhitespaceTokenStream {
                text,
                chars: text.char_indices(),
                token: Token::default(),
            }
        }
    }

    impl<'a> WhitespaceTokenStream<'a> {
        // search for the end of the current token.
        fn search_token_end(&mut self) -> usize {
            (&mut self.chars)
                .filter(|&(_, ref c)| c.is_whitespace())
                .map(|(offset, _)| offset)
                .next()
                .unwrap_or(self.text.len())
        }
    }

    impl<'a> TokenStream for WhitespaceTokenStream<'a> {
        fn advance(&mut self) -> bool {
            self.token.text.clear();
            self.token.position = self.token.position.wrapping_add(1);
            while let Some((offset_from, c)) = self.chars.next() {
                if !c.is_whitespace() {
                    let offset_to = self.search_token_end();
                    self.token.offset_from = offset_from;
                    self.token.offset_to = offset_to;
                    self.token.text.push_str(&self.text[offset_from..offset_to]);
                    return true;
                }
            }
            false
        }

        fn token(&self) -> &Token {
            &self.token
        }

        fn token_mut(&mut self) -> &mut Token {
            &mut self.token
        }
    }

    #[cfg(test)]
    mod tests {
        use tantivy::tokenizer::{TextAnalyzer, Token};

        use super::WhitespaceTokenizer;

        #[test]
        fn test_whitespace_tokenizer_with_unicode_spaces() {
            let tokens = token_stream_helper("わ |　か　　花");
            assert_eq!(tokens.len(), 4);
        }

        fn token_stream_helper(text: &str) -> Vec<Token> {
            let a = TextAnalyzer::from(WhitespaceTokenizer);
            let mut token_stream = a.token_stream(text);
            let mut tokens: Vec<Token> = vec![];
            let mut add_token = |token: &Token| {
                tokens.push(token.clone());
            };
            token_stream.process(&mut add_token);
            tokens
        }
    }
}
