use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::{parse_macro_input, spanned::Spanned, DataEnum, DataStruct, DeriveInput};

#[proc_macro_error]
#[proc_macro_derive(NazmcParse)]
pub fn derive_nazmc_parser(input: TokenStream) -> TokenStream {

    let derive_input = parse_macro_input!(input as DeriveInput);

    let node_name = &derive_input.ident;

    match derive_input.data {
        syn::Data::Struct(data_struct) => derive_for_struct(data_struct),
        syn::Data::Enum(data_enum) => derive_for_enum(data_enum),
        syn::Data::Union(_) => abort!(node_name.span(), "Cannot dervie the trait `NazmcParse` for unions"),
    }
}


fn derive_for_struct(data_struct: DataStruct) -> TokenStream {
    todo!()
}

fn derive_for_enum(data_enum: DataEnum) -> TokenStream {
    todo!()
}