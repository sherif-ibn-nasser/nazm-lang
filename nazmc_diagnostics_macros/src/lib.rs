use owo_colors::OwoColorize;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn nazmc_error_code(attr: TokenStream, input: TokenStream) -> TokenStream {
    
    let ast: syn::DeriveInput = syn::parse(input.clone()).unwrap();

    let id = ast.ident;
    let name = id.to_string();

    if !name.starts_with("E") {
        panic!("The error class should start with the capital letter `E`, found `{}`", name.chars().next().unwrap_or_default())
    }

    let code = &name[1..];
    if code.len() != 4 || !code.chars().all(|c| c.is_ascii_digit()) {
        panic!("The error class should contains the error code wich consists from only 4 digits, found `{}`", code)
    }

    let full_code = "خطأ".to_string()+"["+code+"]";
    let styled_code = format!("{}", full_code.bold().red());

    let mut output : TokenStream = quote! {
        impl ErrorLevel for #id {
            const CODE: &'static str = #styled_code;
        }
    }.into();

    let input_derive_default: TokenStream = "#[derive(Default)]\n".parse().unwrap();

    output.extend(input_derive_default);

    output.extend(input);

    output

}
