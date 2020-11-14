use quote::{quote, ToTokens};
use synstructure::BindStyle;

use crate::attrs::{DeriveFieldFlags, DeriveFlags};
use crate::err::{DeriveError, ResultTokenizer};

pub fn scan_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    ResultTokenizer::new(|| scan_derive_impl(s)).into_token_stream()
}

fn scan_derive_impl(
    mut s: synstructure::Structure,
) -> Result<proc_macro2::TokenStream, DeriveError> {
    let type_flags = DeriveFlags::new(&s.ast().attrs)?;

    let scan_body = generate_scan_body(&type_flags, &mut s);

    let mut res = proc_macro2::TokenStream::new();

    res.extend(s.gen_impl(quote! {
        gen unsafe impl shredder::Scan for @Self {
            fn scan(&self, scanner: &mut shredder::Scanner<'_>) {
                use shredder::Scan;

                match *self { #scan_body }
            }
        }
    }));

    res.extend(s.gen_impl(quote! {
        gen unsafe impl shredder::marker::GcSafe for @Self {}
    }));

    if type_flags.generate_gc_drop {
        res.extend(s.gen_impl(quote! {
            gen unsafe impl shredder::marker::GcDrop for @Self {}
        }));
    }

    if type_flags.generate_gc_deref {
        res.extend(s.gen_impl(quote! {
            gen unsafe impl shredder::marker::GcDeref for @Self {}
        }));
    }

    Ok(res)
}

fn generate_scan_body(
    struct_flags: &DeriveFlags,
    s: &mut synstructure::Structure,
) -> proc_macro2::TokenStream {
    s.bind_with(|_| BindStyle::Ref);

    s.each(|bi| {
        ResultTokenizer::new(|| {
            let field_attrs = DeriveFieldFlags::new(&bi.ast().attrs)?;
            let mut scanning_exprs = proc_macro2::TokenStream::new();

            if !field_attrs.skip_scan && !field_attrs.unsafe_skip_gc_safe {
                let expr = quote! {
                    scanner.scan(#bi);
                };
                scanning_exprs.extend(expr);
            }

            if !field_attrs.unsafe_skip_gc_safe && field_attrs.skip_scan {
                let expr = quote! {
                    shredder::plumbing::check_gc_safe(#bi);
                };
                scanning_exprs.extend(expr);
            }

            if !field_attrs.unsafe_skip_gc_deref && struct_flags.generate_gc_deref {
                let expr = quote! {
                    shredder::plumbing::check_gc_deref(#bi);
                };
                scanning_exprs.extend(expr);
            }

            if !field_attrs.unsafe_skip_gc_drop && struct_flags.generate_gc_drop {
                let expr = quote! {
                    shredder::plumbing::check_gc_drop(#bi);
                };
                scanning_exprs.extend(expr);
            }

            Ok(scanning_exprs)
        })
    })
}
