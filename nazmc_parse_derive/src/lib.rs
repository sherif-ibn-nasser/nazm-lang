use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
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
        syn::Data::Enum(data_enum) => {
            let token_stream = derive_for_enum(tree_name, data_enum);
            if token_stream.to_string().is_empty() {
                return TokenStream::new();
            }
            token_stream
        }
        syn::Data::Struct(data_struct) => {
            let token_stream = derive_for_struct(tree_name, data_struct);
            if token_stream.to_string().is_empty() {
                return TokenStream::new();
            }
            token_stream
        }
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

fn derive_for_enum(enum_name: &Ident, data_enum: DataEnum) -> TokenStream2 {
    let emit_err = |span: &Span2| {
        emit_error!(
            span,
            "Variant must be a tuple with a one field _ or Box<_> where ParseResult<_>: NazmcParse"
        )
    };

    if data_enum.variants.len() <= 1 {
        emit_error!(
            enum_name.span(),
            "Enum must have at least two variants to generate parsing methods on them"
        );
        return quote! {};
    }

    let mut types = vec![];

    for v in data_enum.variants.iter().clone() {
        if v.fields.len() != 1 {
            emit_err(&v.span());
            continue;
        }

        let ty = &v.fields.iter().clone().next().unwrap().ty;

        let ty_extracted = extract_segment_and_generic_args(ty);

        if ty_extracted.is_none() {
            if let Type::Path(TypePath { qself: None, path }) = ty {
                if let Some(_) = path.get_ident() {
                    types.push((ty, false)); // Value is not boxed
                } else {
                    emit_err(&ty.span());
                }
            }
            continue;
        }

        let (segment, args) = ty_extracted.unwrap();

        if segment != "Box" {
            emit_err(&ty.span());
            continue;
        }

        let GenericArgument::Type(ty) = &args[0] else {
            emit_err(&ty.span());
            continue;
        };

        types.push((ty, true)); // Value is boxed
    }

    // Errors occured in fields
    if types.len() != data_enum.variants.len() {
        return quote! {};
    }

    let last_variant_idx = data_enum.variants.len() - 1;

    let mut impl_parse_block = quote! {

        let peek_idx = iter.peek_idx;

    };

    let iter = data_enum.variants.iter().zip(types.iter()).enumerate();

    for (i, (variant, (ty, is_boxed))) in iter {
        let variant_name = &variant.ident;
        let return_tree_stm = if *is_boxed {
            quote! { Box::new(node.tree) }
        } else {
            quote! { node.tree }
        };

        let variant_stm = if i < last_variant_idx {
            quote! {
                if let ParseResult::Parsed(node) = <ParseResult<#ty>>::parse(iter) {
                    return ParseResult::Parsed(
                        SyntaxNode {
                            span: node.span,
                            is_broken: node.is_broken,
                            tree: #enum_name::#variant_name(#return_tree_stm),
                        }
                    );
                }

                iter.peek_idx = peek_idx; // Backtrack
            }
        } else {
            quote! {
                match <ParseResult<#ty>>::parse(iter) {
                    ParseResult::Parsed(node) => {
                        return ParseResult::Parsed(
                            SyntaxNode {
                                span: node.span,
                                is_broken: node.is_broken,
                                tree: #enum_name::#variant_name(#return_tree_stm),
                            }
                        );
                    }
                    ParseResult::Unexpected { span, found, is_start_failure } => {
                        // No backtracking after last variant
                        return ParseResult::Unexpected {
                            span,
                            found,
                            is_start_failure,
                        };
                    }
                }
            }
        };

        impl_parse_block.extend(variant_stm);
    }

    impl_parse_block
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

        match field_ty {
            ParseFieldType::BoxedSyntaxNode(ty) | ParseFieldType::SyntaxNode(ty) => {
                let return_tree_stm = if matches!(field_ty, ParseFieldType::BoxedSyntaxNode(_)) {
                    quote! { Box::new(tree) }
                } else {
                    quote! { tree }
                };
                fields_stms.extend(quote! {
                    let peek_idx = iter.peek_idx;
                    let #field_name = match <ParseResult<#ty>>::parse(iter) {
                        ParseResult::Parsed(tree) => {
                            if tree.is_broken {
                                is_broken = true;
                            }
                            span = Some(span.unwrap_or(tree.span).merged_with(&tree.span));
                            #return_tree_stm
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

                is_start_failure = false; // This will make future SyntaxNode type return `false`

                continue;
            }
            _ => {}
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
            SyntaxNode {
                span: span.unwrap_or(iter.peek_start_span()),
                is_broken: is_broken || span.is_none(),
                tree: #tree_name {
                    #fields_decl_in_struct
                }
            }
        );
    }
}

fn check_field(field: &Field) -> Option<ParseFieldType> {
    let ty = &field.ty;
    extract_field_type(ty).or_else(||{
        emit_error!(
            ty.span(),
            "Field must be one of those types:\n
            SyntaxNode<_> or boxed,\n
            ParseResult<_> or boxed,\n
            Optional<_> or boxed,\n
            Vec<SyntaxNode<_>>,\n
            ZeroOrMany<_,_> or OneOrMany<_,_> where ParseResult<_> : NazmcParse";
            note = "The type should be pure and not in path notation, i.e., ParseResult<_> and not crate::ParseResult<_>";
        );
        None
    })
}

enum ParseFieldType<'a> {
    Boxed(&'a Type),
    BoxedSyntaxNode(&'a Type),
    SyntaxNode(&'a Type),
    ParseResult(&'a Type),
    /// i.e. ZeroOrOne
    Optional(&'a Type),
    VecOfSyntaxNode(&'a Type),
    ZeroOrMany(&'a Type, &'a Type),
    OneOrMany(&'a Type, &'a Type),
}

fn extract_field_type(ty: &Type) -> Option<ParseFieldType> {
    let Some((segment, args)) = extract_segment_and_generic_args(ty) else {
        return None;
    };

    match segment.as_str() {
        "Box" => {
            if args.len() != 1 {
                return None;
            }

            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };

            let Some((segment, args)) = extract_segment_and_generic_args(ty) else {
                return None;
            };

            if args.len() != 1
                || segment != "SyntaxNode" && segment != "ParseResult" && segment != "Optional"
            {
                return None;
            }

            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };

            if segment != "SyntaxNode" {
                return Some(ParseFieldType::Boxed(ty));
            }

            Some(ParseFieldType::BoxedSyntaxNode(ty))
        }
        "SyntaxNode" => {
            if args.len() != 1 {
                return None;
            }
            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };
            Some(ParseFieldType::SyntaxNode(ty))
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

            if args.len() != 1 || segment != "SyntaxNode" {
                return None;
            }

            let GenericArgument::Type(ty) = &args[0] else {
                return None;
            };

            Some(ParseFieldType::VecOfSyntaxNode(ty))
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
