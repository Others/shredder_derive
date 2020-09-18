use proc_macro::TokenStream;

pub struct DeriveError {
    token_stream: TokenStream,
}

impl DeriveError {
    pub fn new(msg: proc_macro2::TokenStream) -> Self {
        Self {
            token_stream: msg.into(),
        }
    }
}

pub fn into_derive_result(result: Result<TokenStream, DeriveError>) -> TokenStream {
    match result {
        Ok(v) => v,
        Err(e) => e.token_stream,
    }
}
