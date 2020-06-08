extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, ToTokens};
use syn::{Data, DataStruct, Field, Fields, Generics, Meta, MetaList, NestedMeta};

#[proc_macro_derive(Scan, attributes(shredder))]
pub fn derive_scan(input: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = derive_input.ident;

    let generics = derive_input.generics;

    match derive_input.data {
        Data::Struct(struct_data) => emit_scan_for_struct(name, generics, struct_data),
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
                span => compile_error!("The `Scan` derive doesn't support unions yet!");
            })
            .into()
        }
    }
}

fn is_shredder_attr(meta_list: &MetaList) -> bool {
    let path = &meta_list.path.segments;
    if path.len() > 1 {
        return false;
    }
    match path.first() {
        Some(seg) => &seg.ident.to_string() == "shredder",
        None => false,
    }
}

fn id_skip(found_skip: &mut bool, found_unsafe_skip: &mut bool, nested_attrs: &NestedMeta) {
    match nested_attrs {
        NestedMeta::Meta(m) => match m {
            Meta::Path(p) => {
                if p.segments.len() != 1 {
                    panic!(
                        "Strange path in `shredder` macro: `{}`",
                        p.segments.to_token_stream()
                    );
                }
                let first = p.segments.first().map(|v| v.ident.to_string());

                if first == Some("skip".to_string()) {
                    *found_skip = true;
                    return;
                }

                if first == Some("unsafe_skip".to_string()) {
                    *found_unsafe_skip = true;
                    return;
                }

                panic!(
                    "Invalid `shredder` flag: `{}`",
                    first.unwrap_or_else(|| "[flag missing]".to_string())
                );
            }
            Meta::List(list) => {
                panic!(
                    "Unknown nested marker in `shredder` macro: `{}`",
                    list.to_token_stream()
                );
            }
            Meta::NameValue(name) => {
                panic!(
                    "Unknown key/value pair in `shredder` macro: `{}`",
                    name.to_token_stream()
                );
            }
        },
        NestedMeta::Lit(lit) => {
            panic!(
                "Strange literal in `shredder` marker macro: `{}`",
                lit.to_token_stream()
            );
        }
    }
}

// TODO: Report errors more elegantly
fn emit_scan_expr<T: ToTokens>(
    field_name: T,
    raw_field: Field,
    scanning_exprs: &mut proc_macro2::TokenStream,
) {
    let mut found_skip = false;
    let mut found_unsafe_skip = false;
    for a in raw_field.attrs {
        if let Ok(Meta::List(meta_list)) = a.parse_meta() {
            if is_shredder_attr(&meta_list) {
                for nested_attrs in &meta_list.nested {
                    id_skip(&mut found_skip, &mut found_unsafe_skip, nested_attrs)
                }
            }
        }
    }

    if found_unsafe_skip {
        return;
    }

    let expr = if found_skip {
        quote! {
            scanner.check_gc_safe(&self.#field_name);
        }
    } else {
        quote! {
            scanner.scan(&self.#field_name);
        }
    };

    scanning_exprs.extend(expr);
}

fn emit_scan_for_struct(name: Ident, generics: Generics, struct_data: DataStruct) -> TokenStream {
    let mut res = proc_macro2::TokenStream::new();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // This is safe, as the `Scan` impl will fail to compile if the fields are not `GcSafe`
    // And `GcSafe` is structural
    let gc_safe_impl = quote! {
        unsafe impl #impl_generics shredder::GcSafe for #name #ty_generics #where_clause  {}
    };
    res.extend(gc_safe_impl);

    let mut scanning_exprs = proc_macro2::TokenStream::new();
    match struct_data.fields {
        Fields::Named(named_fields) => {
            for f in named_fields.named {
                let field_name = f.ident.clone().expect("Name fields must have a name...");
                emit_scan_expr(field_name, f, &mut scanning_exprs);
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            for (i, f) in unnamed_fields.unnamed.into_iter().enumerate() {
                let idx = syn::Index::from(i);
                emit_scan_expr(idx, f, &mut scanning_exprs);
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

    res.into()
}
