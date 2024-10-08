use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote;
use syn::PathArguments;
use syn::{
    parse_macro_input, spanned::Spanned, DataEnum, DataStruct, DeriveInput, Field, GenericArgument,
    Ident, Type, TypePath,
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

        let ty_checked = check_type(ty);

        let pair = match ty_checked {
            Some(ParseFieldType::Boxed(ty)) => (ty, true),
            Some(ParseFieldType::Pure(ty)) => (ty, false),
            _ => {
                emit_err(&ty.span());
                continue;
            }
        };

        types.push(pair);
    }

    // Errors occured in fields
    if types.len() != data_enum.variants.len() {
        return quote! {};
    }

    let last_variant_idx = data_enum.variants.len() - 1;

    let mut impl_spanned_block_match_guards: TokenStream2 = quote! {};

    let mut impl_is_broken_block_match_guards: TokenStream2 = quote! {};

    let mut impl_parse_block = quote! {
        let peek_idx = iter.peek_idx;
    };

    let iter = data_enum.variants.iter().zip(types.iter()).enumerate();

    for (i, (variant, (ty, is_boxed))) in iter {
        let variant_name = &variant.ident;

        impl_spanned_block_match_guards.extend(quote! {
            Self::#variant_name(tree) => tree.span(),
        });

        impl_is_broken_block_match_guards.extend(quote! {
            Self::#variant_name(tree) => tree.is_broken(),
        });

        let return_tree_stm = if *is_boxed {
            quote! { Box::new(tree) }
        } else {
            quote! { tree }
        };

        let variant_stm = if i < last_variant_idx {
            quote! {
                if let Ok(tree) = <ParseResult<#ty>>::parse(iter) {
                    return Ok(#enum_name::#variant_name(#return_tree_stm));
                }

                iter.peek_idx = peek_idx; // Backtrack
            }
        } else {
            quote! {
                match <ParseResult<#ty>>::parse(iter) {
                    Ok(tree) => {
                        return Ok(#enum_name::#variant_name(#return_tree_stm));
                    }
                    Err(err) => {
                        // No backtracking after last variant
                        return Err(err);
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

    if fields_types.any(|op| op.is_none()) {
        return quote! {};
    }

    let fields_types = fields_types_cloned.map(|field_ty| field_ty.unwrap());

    let fields_zipped = data_struct.fields.iter().zip(fields_types);

    let mut fields_parse_stms: TokenStream2 = quote! {};

    let mut fields_decl_in_struct: TokenStream2 = quote! {};

    for (field, field_ty) in fields_zipped {
        let field_name = field.ident.clone().unwrap();

        fields_decl_in_struct.extend(quote! {
            #field_name: #field_name,
        });

        let field_parse_stm = match field_ty {
            ParseFieldType::Pure(ty) | ParseFieldType::Boxed(ty) => {
                let return_tree_stm = if matches!(field_ty, ParseFieldType::Boxed(_)) {
                    quote! { Box::new(tree) }
                } else {
                    quote! { tree }
                };
                quote! {
                    let peek_idx = iter.peek_idx;
                    let #field_name = match <ParseResult<#ty>>::parse(iter) {
                        Ok(tree) => #return_tree_stm,
                        Err(err) =>{
                            iter.peek_idx = peek_idx; // Backtrack
                            return Err(err);
                        }
                    };
                }
            }
            _ => {
                let field_ty_token = field.ty.clone();

                quote! {
                    let #field_name = <#field_ty_token>::parse(iter);
                }
            }
        };

        fields_parse_stms.extend(field_parse_stm);
    }

    let impl_parse_block = quote! {
        #fields_parse_stms
        return Ok(
            #tree_name {
                #fields_decl_in_struct
            }
        );
    };

    impl_parse_block
}

fn check_field(field: &Field) -> Option<ParseFieldType> {
    let ty = &field.ty;
    check_type(ty).or_else(||{
        emit_error!(
            ty.span(),
            "Field must be one of those types:\n
            _,\n
            ParseResult<_>,\n
            Option<_>,\n
            Vec<_>,\n
            ZeroOrMany<_,_> or OneOrMany<_,_> where ParseResult<_> : NazmcParse";
            note = "The type should be pure and not in path notation, i.e., ParseResult<_> and not crate::ParseResult<_>";
        );
        None
    })
}

enum ParseFieldType<'a> {
    Pure(&'a Type),
    Boxed(&'a Type),
    ParseResult(&'a Type),
    /// i.e. ZeroOrOne
    Option(&'a Type),
    Vec(&'a Type),
    ZeroOrMany(&'a Type, &'a Type),
    OneOrMany(&'a Type, &'a Type),
}

fn check_type(ty: &Type) -> Option<ParseFieldType> {
    match &ty {
        // Case 1: Pure Id
        Type::Path(TypePath { qself: None, path }) if path.get_ident().is_some() => {
            Some(ParseFieldType::Pure(&ty))
        }

        // Case 2: Wrapped types
        Type::Path(type_path) => {
            if let Some(inner) = is_wrapped_type(type_path, "Box") {
                Some(ParseFieldType::Boxed(inner))
            } else if let Some(inner) = is_wrapped_type(type_path, "ParseResult") {
                Some(ParseFieldType::ParseResult(inner))
            } else if let Some(inner) = is_wrapped_type(type_path, "Option") {
                Some(ParseFieldType::Option(inner))
            } else if let Some(inner) = is_wrapped_type(type_path, "Vec") {
                Some(ParseFieldType::Vec(inner))
            } else if let Some((id1, id2)) = is_two_wrapped_types(type_path, "ZeroOrMany") {
                Some(ParseFieldType::ZeroOrMany(id1, id2))
            } else if let Some((id1, id2)) = is_two_wrapped_types(type_path, "OneOrMany") {
                Some(ParseFieldType::OneOrMany(id1, id2))
            } else {
                None
            }
        }
        _ => None, // Unhandled types
    }
}

fn is_wrapped_type<'a>(type_path: &'a TypePath, wrapper: &'a str) -> Option<&'a Type> {
    // Ensure there is only one path segment
    if type_path.path.segments.len() != 1 {
        return None;
    }

    let segment = &type_path.path.segments[0];

    // Check if the wrapper matches (e.g., "Box", "Option", etc.)
    if segment.ident != wrapper {
        return None;
    }

    // Ensure we have angle bracketed arguments (like `<T>`)
    if let PathArguments::AngleBracketed(ref args) = segment.arguments {
        // Ensure exactly one generic argument
        if args.args.len() != 1 {
            return None;
        }

        // Return the inner type if it's a valid `Type`
        if let GenericArgument::Type(inner_type) = &args.args[0] {
            return Some(inner_type);
        }
    }

    None
}

fn is_two_wrapped_types<'a>(
    type_path: &'a TypePath,
    wrapper: &'a str,
) -> Option<(&'a Type, &'a Type)> {
    // Ensure there is only one path segment
    if type_path.path.segments.len() != 1 {
        return None;
    }

    let segment = &type_path.path.segments[0];

    // Check if the wrapper matches (e.g., "ZeroOrMany", "OneOrMany", etc.)
    if segment.ident != wrapper {
        return None;
    }

    // Ensure we have angle bracketed arguments (like `<T, U>`)
    if let PathArguments::AngleBracketed(ref args) = segment.arguments {
        // Ensure exactly two generic arguments
        if args.args.len() != 2 {
            return None;
        }

        // Extract and return both generic arguments if they are valid `Type`s
        if let (GenericArgument::Type(first), GenericArgument::Type(second)) =
            (&args.args[0], &args.args[1])
        {
            return Some((first, second));
        }
    }

    None
}
