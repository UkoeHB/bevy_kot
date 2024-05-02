//local shortcuts

//third-party shortcuts
//use bevy_macro_utils::get_struct_fields;

//standard shortcuts
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

use proc_macro::Span;
use syn::{Data, DataStruct, Error, Fields, FieldsNamed};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Get the fields of a data structure if that structure is a struct with named fields;
/// otherwise, return a compile error that points to the site of the macro invocation.
//stolen from bevy_macro_utils for migration to bevy v0.13
//https://github.com/bevyengine/bevy/commit/eeb0c2f2e4f0cdae262d4c3393c0aaa28deb09ce#diff-5555d250ff0f16b55c69bbfd89e6ba879ded9734ad32adf4a7135457b652e38c
fn get_named_struct_fields(data: &syn::Data) -> syn::Result<&FieldsNamed> {
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => Ok(fields),
        _ => Err(Error::new(
            // This deliberately points to the call site rather than the structure
            // body; marking the entire body as the source of the error makes it
            // impossible to figure out which `derive` has a problem.
            Span::call_site().into(),
            "Only structs with named fields are supported",
        )),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_style_impl(input: TokenStream) -> TokenStream
{
    let mut ast = parse_macro_input!(input as DeriveInput);
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    TokenStream::from(quote! {
        impl #impl_generics ::bevy_kot_ui::Style for #struct_name #ty_generics #where_clause {}
    })
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_style_bundle_impl(input: TokenStream) -> TokenStream
{
    // parse struct members
    let ast = parse_macro_input!(input as DeriveInput);

    let named_fields = match get_named_struct_fields(&ast.data)
    {
        Ok(fields) => &fields.named,
        Err(e) => return e.into_compile_error().into(),
    };

    // prepare per-member code statements
    let mut field_get_styles = Vec::new();
    for field in named_fields.iter().map(|field| field.ident.as_ref().unwrap())
    {
        field_get_styles.push(quote! { self.#field.get_styles(&mut *func); });
    }

    // unpack code statements into trait implementation
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let struct_name = &ast.ident;

    TokenStream::from(quote! {
        impl #impl_generics ::bevy_kot_ui::StyleBundle for #struct_name #ty_generics #where_clause
        {
            #[allow(unused_variables)]
            #[inline]
            fn get_styles(self, func: &mut impl FnMut(std::sync::Arc<dyn std::any::Any + Send + Sync + 'static>))
            {
                #(#field_get_styles)*
            }
        }
    })
}

//-------------------------------------------------------------------------------------------------------------------
