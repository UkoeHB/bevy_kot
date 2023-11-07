//local shortcuts

//third-party shortcuts
use bevy_macro_utils::get_named_struct_fields;

//standard shortcuts
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

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
        impl #impl_generics ::bevy_kot::ui::Style for #struct_name #ty_generics #where_clause {}
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
        impl #impl_generics ::bevy_kot::ui::StyleBundle for #struct_name #ty_generics #where_clause
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
