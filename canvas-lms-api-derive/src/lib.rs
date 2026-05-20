use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro that generates a `req()` helper method returning `&Arc<Requester>`.
///
/// The struct must contain a field `requester: Option<Arc<Requester>>` (with
/// `#[serde(skip)]`). The macro generates:
///
/// ```ignore
/// fn req(&self) -> &Arc<crate::http::Requester> {
///     self.requester.as_ref().expect("requester not initialized")
/// }
/// ```
#[proc_macro_derive(CanvasResource)]
pub fn derive_canvas_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn req(&self) -> &std::sync::Arc<crate::http::Requester> {
                self.requester.as_ref().expect("requester not initialized")
            }
        }
    };

    TokenStream::from(expanded)
}
