use std::str::FromStr;

use proc_macro::{Span, TokenStream};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::ToTokens;
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Else},
    DataEnum, DataStruct, DeriveInput, Field, GenericArgument, Type, TypePath,
};

#[proc_macro_error]
#[proc_macro_derive(NazmcParse)]
pub fn derive_nazmc_parser(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let node_name = &derive_input.ident;

    match derive_input.data {
        syn::Data::Struct(data_struct) => derive_for_struct(data_struct),
        syn::Data::Enum(data_enum) => derive_for_enum(data_enum),
        syn::Data::Union(_) => abort!(
            node_name.span(),
            "Cannot dervie the trait `NazmcParse` for unions"
        ),
    }
}

fn derive_for_struct(data_struct: DataStruct) -> TokenStream {
    let mut out: TokenStream = TokenStream::new();
    for (i, field) in data_struct.fields.iter().enumerate() {
        check_field(field);
    }
    out
}

fn derive_for_enum(data_enum: DataEnum) -> TokenStream {
    todo!()
}

fn check_field(field: &Field) {
    let ty = &field.ty;
    let Some(field_type) = extract_field_type(ty) else {
        emit_error!(
            ty.span(),
            "Field must be one of those types: ParseResult<_>, Optional<_>, Vec<ASTNode<_>>, ZeroOrMany<_,_>, OneOrMany<_,_> where ParseResult<_> : NazmcParse";
            note = "The type should be pure and not in path notation, i.e., ParseResult<_> and not crate::ParseResult<_>";
        );
        return;
    };
}

enum ParseFieldType<'a> {
    ParseResult(&'a Type),
    /// i.e. ZeroOrOne
    Optional(&'a Type),
    VecOfASTNode(&'a Type),
    ZeroOrMany(&'a Type, &'a Type),
    OneOrMany(&'a Type, &'a Type),
}

fn extract_field_type(ty: &Type) -> Option<ParseFieldType> {
    let Some((segment, args)) = extract_segment_and_generic_args(ty) else {
        return None;
    };

    match segment.as_str() {
        "ParseResult" => {
            if args.len() != 1 {
                return None;
            }
            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };
            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };
            Some(ParseFieldType::ParseResult(ty))
        }
        "Optional" => {
            if args.len() != 1 {
                return None;
            }
            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };
            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };
            Some(ParseFieldType::Optional(ty))
        }
        "Vec" => {
            if args.len() != 1 {
                return None;
            }

            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };

            let Some((segment, args)) = extract_segment_and_generic_args(ty) else {
                return None;
            };

            if args.len() != 1 || segment != "ASTNode" {
                return None;
            }

            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };

            Some(ParseFieldType::VecOfASTNode(ty))
        }
        "ZeroOrMany" => {
            if args.len() != 2 {
                return None;
            }
            let GenericArgument::Type(ty0) = &args[0] else {
                return None;
            };
            let GenericArgument::Type(ty1) = &args[1] else {
                return None;
            };
            Some(ParseFieldType::ZeroOrMany(ty0, ty1))
        }
        "OneOrMany" => {
            if args.len() != 2 {
                return None;
            }
            let GenericArgument::Type(ty0) = &args[0] else {
                return None;
            };
            let GenericArgument::Type(ty1) = &args[1] else {
                return None;
            };
            Some(ParseFieldType::OneOrMany(ty0, ty1))
        }
        _ => None,
    }
}

fn extract_segment_and_generic_args(
    ty: &Type,
) -> Option<(String, &Punctuated<GenericArgument, Comma>)> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };

    let segment = path.segments[0].ident.to_string();

    match &path.segments[0].arguments {
        syn::PathArguments::AngleBracketed(args) => Some((segment, &args.args)),
        _ => None,
    }
}
