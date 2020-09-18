use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Attribute, Meta, NestedMeta};

use crate::err::DeriveError;

pub struct DeriveFlags {
    pub generate_gc_deref: bool,
    pub generate_gc_drop: bool,
}

const GENERATE_GC_DEREF: &str = "can_deref";
const DONT_GENERATE_GC_DROP: &str = "cant_drop";

impl DeriveFlags {
    pub fn new(attrs: &[Attribute]) -> Result<Self, DeriveError> {
        let mut generate_gc_deref = false;
        let mut dont_generate_gc_drop = false;

        for at in attrs {
            let shredder_flags = find_shredder_flags(at)?;
            for (flag, span) in shredder_flags {
                match flag.trim() {
                    GENERATE_GC_DEREF => {
                        if generate_gc_deref {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            generate_gc_deref = true;
                        }
                    }
                    DONT_GENERATE_GC_DROP => {
                        if dont_generate_gc_drop {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            dont_generate_gc_drop = true;
                        }
                    }
                    _ => {
                        return Err(DeriveError::new(quote_spanned! {
                            span => compile_error!("Unknown shredder flag");
                        }));
                    }
                }
            }
        }

        Ok(Self {
            generate_gc_deref,
            generate_gc_drop: !dont_generate_gc_drop,
        })
    }
}

pub struct DeriveFieldFlags {
    pub skip_scan: bool,
    pub unsafe_skip_gc_deref: bool,
    pub unsafe_skip_gc_drop: bool,
    pub unsafe_skip_gc_safe: bool,
}

const SKIP_SCAN: &str = "skip_scan";
const UNSAFE_SKIP_GC_DEREF: &str = "unsafe_skip_gc_deref";
const UNSAFE_SKIP_GC_DROP: &str = "unsafe_skip_gc_drop";
const UNSAFE_SKIP_GC_SAFE: &str = "unsafe_skip_gc_safe";
const UNSAFE_SKIP_ALL: &str = "unsafe_skip_all";

impl DeriveFieldFlags {
    pub fn new(attrs: &[Attribute]) -> Result<Self, DeriveError> {
        let mut skip_scan = false;
        let mut unsafe_skip_gc_deref = false;
        let mut unsafe_skip_gc_drop = false;
        let mut unsafe_skip_gc_safe = false;
        let mut unsafe_skip_all = false;

        for at in attrs {
            let shredder_flags = find_shredder_flags(at)?;
            for (flag, span) in shredder_flags {
                match flag.as_str() {
                    SKIP_SCAN => {
                        if skip_scan {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            skip_scan = true;
                        }
                    }
                    UNSAFE_SKIP_GC_DEREF => {
                        if unsafe_skip_gc_deref {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            unsafe_skip_gc_deref = true;
                        }
                    }

                    UNSAFE_SKIP_GC_DROP => {
                        if unsafe_skip_gc_drop {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            unsafe_skip_gc_drop = true;
                        }
                    }

                    UNSAFE_SKIP_GC_SAFE => {
                        if unsafe_skip_gc_safe {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            unsafe_skip_gc_safe = true;
                        }
                    }

                    UNSAFE_SKIP_ALL => {
                        if unsafe_skip_all {
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Duplicate shredder flag");
                            }));
                        } else {
                            unsafe_skip_all = true;
                        }
                    }
                    _ => {
                        return Err(DeriveError::new(quote_spanned! {
                            span => compile_error!("Unknown shredder flag");
                        }));
                    }
                }
            }
        }

        if unsafe_skip_all {
            skip_scan = true;
            unsafe_skip_gc_deref = true;
            unsafe_skip_gc_drop = true;
            unsafe_skip_gc_safe = true;
        }

        Ok(Self {
            skip_scan,
            unsafe_skip_gc_deref,
            unsafe_skip_gc_drop,
            unsafe_skip_gc_safe,
        })
    }
}

fn find_shredder_flags(attr: &Attribute) -> Result<Vec<(String, Span)>, DeriveError> {
    let meta = attr.parse_meta().expect("attribute should parse correctly");

    let mut shredder_flags = Vec::new();

    if let Meta::List(p) = meta {
        let segments = p.path.segments;
        if let Some(start) = segments.first() {
            if &start.ident.to_string() == "shredder" {
                if segments.len() > 1 {
                    return Err(DeriveError::new(quote_spanned! {
                        segments.span() => compile_error!("Unknown path shredder::???");
                    }));
                }

                let nested = p.nested;
                for n in nested {
                    if let NestedMeta::Meta(m) = n {
                        if let Meta::Path(p) = m {
                            let v: Vec<String> =
                                p.segments.iter().map(|s| s.ident.to_string()).collect();
                            shredder_flags.push((v.join("::"), segments.span()));
                        } else {
                            let span = m.span();
                            return Err(DeriveError::new(quote_spanned! {
                                span => compile_error!("Unknown shredder flag");
                            }));
                        }
                    } else {
                        let span = n.span();
                        return Err(DeriveError::new(quote_spanned! {
                            span => compile_error!("Unknown shredder flag");
                        }));
                    }
                }
            }
        }
    }

    Ok(shredder_flags)
}
