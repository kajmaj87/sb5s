use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(NumericId)]
pub fn numeric_id_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed,
            _ => panic!("NumericId can only be derived for structs with a single unnamed field"),
        },
        _ => panic!("NumericId can only be derived for structs"),
    };

    let field_type = &fields.first().unwrap().ty;

    // Generate the implementation with a more flexible path
    let expanded = quote! {
        impl NumericId for #name {
            fn value(&self) -> #field_type {
                self.0
            }

            fn from_value(value: #field_type) -> Self {
                #name(value)
            }
        }
    };

    TokenStream::from(expanded)
}
