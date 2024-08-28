use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, DataEnum,
    DataStruct, DeriveInput, Field, GenericArgument, Ident, Type, TypePath,
};

#[proc_macro_error]
#[proc_macro_derive(NazmcParse)]
pub fn derive_nazmc_parser(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let tree_name = &derive_input.ident;

    let impl_parse_block = match derive_input.data {
        syn::Data::Struct(data_struct) => derive_for_struct(tree_name, data_struct),
        syn::Data::Enum(data_enum) => derive_for_enum(data_enum),
        syn::Data::Union(_) => abort!(
            tree_name.span(),
            "Cannot dervie the trait `NazmcParse` for unions"
        ),
    };

    quote! {
        impl NazmcParse for ParseResult<#tree_name> {

            fn parse(iter: &mut TokensIter) -> Self {
                #impl_parse_block
            }
        }
    }
    .into()
}

fn derive_for_struct(tree_name: &Ident, data_struct: DataStruct) -> TokenStream2 {
    let mut fields_types = data_struct
        .fields
        .iter()
        .clone()
        .map(|field| check_field(&field));

    // Clone the iterator to use it again later
    let fields_types_cloned = fields_types.clone();

    if !fields_types.all(|op| op.is_some()) {
        return quote! {};
    }

    let fields_types = fields_types_cloned.map(|field_ty| field_ty.unwrap());

    let fields_zipped = data_struct.fields.iter().zip(fields_types);

    let span_and_is_broken_stms = quote! {
        let mut span = None; // The span of this node
        let mut is_broken = false;  // True if at least one child is broken
    };

    let mut fields_stms: TokenStream2 = quote! {};

    let mut fields_decl_in_struct: TokenStream2 = quote! {};

    let mut is_start_failure = true;

    // TODO: Optimze and remove span as Option when it is surely determined
    for (field, field_ty) in fields_zipped {
        let field_name = field.ident.clone().unwrap();

        fields_decl_in_struct.extend(quote! {
            #field_name: #field_name,
        });

        if let ParseFieldType::ASTNode(ty) = field_ty {
            fields_stms.extend(quote! {
                let peek_idx = iter.peek_idx;
                let #field_name = match <ParseResult<#ty>>::parse(iter) {
                    ParseResult::Parsed(tree) => {
                        if tree.is_broken {
                            is_broken = true;
                        }
                        span = Some(span.unwrap_or(tree.span).merged_with(&tree.span));
                        tree
                    },
                    ParseResult::Unexpected { span, found, .. } =>{
                        iter.peek_idx = peek_idx;
                        return ParseResult::Unexpected {
                            span,
                            found,
                            is_start_failure: #is_start_failure,
                        };
                    }
                };
            });

            is_start_failure = false; // This will make future ASTNode type return `false`

            continue;
        }

        let field_ty_token = field.ty.clone();

        let field_parsing_instruction = quote! {
            let #field_name = <#field_ty_token>::parse(iter);
            if !#field_name.is_parsed_and_valid() {
                is_broken = true;
            }
            if let Some(field_span) = #field_name.span() {
                span = Some(span.unwrap_or(field_span).merged_with(&field_span));
            }
        };

        fields_stms.extend(field_parsing_instruction);
    }

    quote! {
        #span_and_is_broken_stms
        #fields_stms
        return Self::Parsed(
            ASTNode {
                span: span.unwrap_or(iter.peek_start_span()),
                is_broken: is_broken || span.is_none(),
                tree: #tree_name {
                    #fields_decl_in_struct
                }
            }
        );
    }
}

fn derive_for_enum(data_enum: DataEnum) -> TokenStream2 {
    todo!()
}

fn check_field(field: &Field) -> Option<ParseFieldType> {
    let ty = &field.ty;
    extract_field_type(ty).or_else(||{
        emit_error!(
            ty.span(),
            "Field must be one of those types:\n ASTNode<_>,\n ParseResult<_>,\n Optional<_>,\n Vec<ASTNode<_>>,\n ZeroOrMany<_,_> or OneOrMany<_,_> where ParseResult<_> : NazmcParse";
            note = "The type should be pure and not in path notation, i.e., ParseResult<_> and not crate::ParseResult<_>";
        );
        None
    })
}

enum ParseFieldType<'a> {
    ASTNode(&'a Type),
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
        "ASTNode" => {
            if args.len() != 1 {
                return None;
            }
            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };
            Some(ParseFieldType::ASTNode(ty))
        }
        "ParseResult" => {
            if args.len() != 1 {
                return None;
            }
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
