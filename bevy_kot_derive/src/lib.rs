//module tree
mod react;
mod style;

//proc exports
use proc_macro::TokenStream;

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(ReactComponent)]
pub fn derive_react_component(input: TokenStream) -> TokenStream
{
    react::derive_react_component_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(ReactResource)]
pub fn derive_react_resource(input: TokenStream) -> TokenStream
{
    react::derive_react_resource_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(Style)]
pub fn derive_style(input: TokenStream) -> TokenStream
{
    style::derive_style_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(StyleBundle)]
pub fn derive_style_bundle(input: TokenStream) -> TokenStream
{
    style::derive_style_bundle_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------
