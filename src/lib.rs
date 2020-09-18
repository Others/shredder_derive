mod attrs;
mod err;

extern crate proc_macro;

use crate::attrs::{DeriveFieldFlags, DeriveFlags};
use crate::err::{into_derive_result, DeriveError};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, ToTokens};
use syn::{Attribute, Data, DataStruct, Field, Fields, Generics};

#[proc_macro_derive(Scan, attributes(shredder))]
pub fn derive_scan(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = derive_input.ident;
    let generics = derive_input.generics;
    let attrs = derive_input.attrs;

    match derive_input.data {
        Data::Struct(struct_data) => {
            into_derive_result(emit_scan_for_struct(name, generics, attrs, struct_data))
        }
        Data::Enum(enum_data) => {
            let span = enum_data.enum_token.span;
            (quote_spanned! {
                span => compile_error!("The `Scan` derive doesn't support enums yet!");
            })
            .into()
        }
        Data::Union(union_data) => {
            let span = union_data.union_token.span;
            (quote_spanned! {
                span => compile_error!("Unions cannot have their scan generated automatically!");
            })
            .into()
        }
    }
}

fn emit_scan_for_struct(
    name: Ident,
    generics: Generics,
    attrs: Vec<Attribute>,
    struct_data: DataStruct,
) -> Result<TokenStream, DeriveError> {
    let mut res = proc_macro2::TokenStream::new();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let scan_flags = DeriveFlags::new(&attrs)?;

    let gc_safe_impl = quote! {
        unsafe impl #impl_generics shredder::marker::GcSafe for #name #ty_generics #where_clause  {}
    };
    res.extend(gc_safe_impl);

    if scan_flags.generate_gc_deref {
        let gc_deref_impl = quote! {
            unsafe impl #impl_generics shredder::marker::GcDeref for #name #ty_generics #where_clause  {}
        };
        res.extend(gc_deref_impl);
    }

    if scan_flags.generate_gc_drop {
        let gc_drop_impl = quote! {
            unsafe impl #impl_generics shredder::marker::GcDrop for #name #ty_generics #where_clause  {}
        };
        res.extend(gc_drop_impl);
    }

    let mut scanning_exprs = proc_macro2::TokenStream::new();
    match struct_data.fields {
        Fields::Named(named_fields) => {
            for f in named_fields.named {
                let field_name = f.ident.clone().expect("Name fields must have a name...");
                emit_scan_expr(field_name, f, &mut scanning_exprs, &scan_flags)?;
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            for (i, f) in unnamed_fields.unnamed.into_iter().enumerate() {
                let idx = syn::Index::from(i);
                emit_scan_expr(idx, f, &mut scanning_exprs, &scan_flags)?;
            }
        }
        Fields::Unit => {}
    }

    let gc_impl = quote! {
        unsafe impl #impl_generics shredder::Scan for #name #ty_generics #where_clause {
            fn scan(&self, scanner: &mut shredder::Scanner) {
                #scanning_exprs
            }
        }
    };
    res.extend(gc_impl);

    Ok(res.into())
}

fn emit_scan_expr<T: ToTokens>(
    field_name: T,
    raw_field: Field,
    scanning_exprs: &mut proc_macro2::TokenStream,
    struct_flags: &DeriveFlags,
) -> Result<(), DeriveError> {
    let field_attrs = DeriveFieldFlags::new(&raw_field.attrs)?;

    if !field_attrs.skip_scan && !field_attrs.unsafe_skip_gc_safe {
        let expr = quote! {
            scanner.scan(&self.#field_name);
        };
        scanning_exprs.extend(expr);
    }

    if !field_attrs.unsafe_skip_gc_safe && field_attrs.skip_scan {
        let expr = quote! {
            shredder::plumbing::check_gc_safe(&self.#field_name);
        };
        scanning_exprs.extend(expr);
    }

    if !field_attrs.unsafe_skip_gc_deref && struct_flags.generate_gc_deref {
        let expr = quote! {
            shredder::plumbing::check_gc_deref(&self.#field_name);
        };
        scanning_exprs.extend(expr);
    }

    if !field_attrs.unsafe_skip_gc_drop && struct_flags.generate_gc_drop {
        let expr = quote! {
            shredder::plumbing::check_gc_drop(&self.#field_name);
        };
        scanning_exprs.extend(expr);
    }

    Ok(())
}
