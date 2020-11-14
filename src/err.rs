use proc_macro2::TokenStream;
use quote::ToTokens;

pub struct DeriveError {
    token_stream: proc_macro2::TokenStream,
}

impl DeriveError {
    pub fn new(msg: proc_macro2::TokenStream) -> Self {
        Self { token_stream: msg }
    }
}

pub struct ResultTokenizer {
    res: Result<proc_macro2::TokenStream, DeriveError>,
}

impl ResultTokenizer {
    pub fn new<F: FnOnce() -> Result<proc_macro2::TokenStream, DeriveError>>(f: F) -> Self {
        Self { res: f() }
    }
}

impl ToTokens for ResultTokenizer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let v = match &self.res {
            Ok(v) => v.to_token_stream(),
            Err(e) => e.token_stream.clone(),
        };
        tokens.extend(v);
    }
}
