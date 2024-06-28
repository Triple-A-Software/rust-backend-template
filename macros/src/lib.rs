use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, ExprPath};

#[proc_macro_derive(JsonErrorResponse, attributes(status_code))]
pub fn derive_error_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    match input.data {
        Data::Enum(ast) => {
            let match_arms = ast.variants.iter().map(|variant| {
                let ident = &variant.ident;
                //dbg!(&variant);
                let fields = variant.fields.iter();
                let fields = if variant.fields.is_empty() {
                    quote! {}
                } else {
                    let fields = fields.map(|_| format_ident!("_"));
                    quote! {
                        (#(#fields),*)
                    }
                };
                let status_code = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.meta.path().is_ident("status_code")) // check if this
                    // attribute is the `status_code` attribute to set the status code
                    .map(|status_code| {
                        let args: ExprPath = status_code.parse_args().expect(
                            "#[status_code(...)] only allows axum `StatusCode::...` expressions",
                        );
                        args.to_token_stream()
                    });
                quote! {
                    #name::#ident #fields => (
                        #status_code,
                        Json(ErrorResponse {
                            error_message: self.to_string(),
                            ..Default::default()
                        }),
                    )
                        .into_response()
                }
            });
            let expanded = quote! {
                impl IntoResponse for #name {
                    fn into_response(self) -> axum::response::Response {
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }
        _ => unimplemented!("This macro is only implemented for enums"),
    }
}
