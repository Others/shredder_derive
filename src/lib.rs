pub(crate) mod attrs;
pub(crate) mod err;
pub(crate) mod finalize_derive;
pub(crate) mod scan_derive;

extern crate proc_macro;

synstructure::decl_derive!([Scan, attributes(shredder)] => scan_derive::scan_derive);
synstructure::decl_derive!([Finalize, attributes(shredder)] => finalize_derive::finalize_derive);
synstructure::decl_derive!([FinalizeFields, attributes(shredder)] => finalize_derive::finalize_fields_derive);
