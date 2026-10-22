use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitInt, parse_macro_input};

#[proc_macro_derive(ScheduleLabel, attributes(schedule))]
pub fn derive_schedule_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut priority: isize = 0;
    for attr in &input.attrs {
        if !attr.path().is_ident("schedule") {
            continue;
        }
        let parsed = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("priority") {
                let value: LitInt = meta.value()?.parse()?;
                priority = value.base10_parse()?;
                Ok(())
            } else {
                Err(meta.error("Unsupported schedule attribute"))
            }
        });
        if let Err(e) = parsed {
            return e.to_compile_error().into();
        }
    }

    let expanded = quote! {
        impl ScheduleLabel for #name {
            fn priority() -> isize {
                #priority
            }
        }
    };

    expanded.into()
}
