use quote::{quote, ToTokens};
use synstructure::BindStyle;

use crate::attrs::{DeriveFieldFlags, DeriveFlags};
use crate::err::{DeriveError, ResultTokenizer};

pub fn finalize_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    ResultTokenizer::new(|| finalize_derive_impl(s)).into_token_stream()
}

fn finalize_derive_impl(
    mut s: synstructure::Structure,
) -> Result<proc_macro2::TokenStream, DeriveError> {
    // We don't use type flags right now, but we create them so the validation logic runs
    let _type_flags = DeriveFlags::new(&s.ast().attrs)?;

    let body = generate_finalize_body(&mut s);

    Ok(s.gen_impl(quote! {
        gen unsafe impl shredder::Finalize for @Self {
            unsafe fn finalize(&mut self) {
                use shredder::Finalize;

                match *self { #body }
            }
        }
    }))
}

pub fn finalize_fields_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    ResultTokenizer::new(|| finalize_fields_derive_impl(s)).into_token_stream()
}

fn finalize_fields_derive_impl(
    mut s: synstructure::Structure,
) -> Result<proc_macro2::TokenStream, DeriveError> {
    // We don't use type flags right now, but we create them so the validation logic runs
    let _type_flags = DeriveFlags::new(&s.ast().attrs)?;

    let body = generate_finalize_body(&mut s);

    Ok(s.gen_impl(quote! {
        gen unsafe impl shredder::FinalizeFields for @Self {
            unsafe fn finalize_fields(&mut self) {
                use shredder::Finalize;

                match *self { #body }
            }
        }
    }))
}

fn generate_finalize_body(s: &mut synstructure::Structure) -> proc_macro2::TokenStream {
    // This feels like bad design...
    s.bind_with(|_| BindStyle::RefMut);

    s.each(|bi| {
        ResultTokenizer::new(|| {
            let field_flags = DeriveFieldFlags::new(&bi.ast().attrs)?;
            if !field_flags.skip_finalize {
                Ok(quote! {
                    (*#bi).finalize();
                })
            } else {
                Ok(quote! {})
            }
        })
    })
}
