use proc_macro::TokenStream;
use quote::quote;

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}

#[proc_macro_attribute]
pub fn task(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(input) => input,
        Err(error) => return token_stream_with_error(item, error),
    };

    if input.sig.asyncness.is_none() {
        return token_stream_with_error(
            item,
            syn::Error::new_spanned(
                input.sig,
                "the #[task] attribute can only be applied to an async function",
            ),
        );
    }

    if input.sig.inputs.len() != 1 {
        return token_stream_with_error(
            item,
            syn::Error::new_spanned(
                input.sig,
                "the #[task] attribute can only be applied to a function with a single argument",
            ),
        );
    }

    let input_fn = input.sig.ident.clone();

    let output = quote! {
        #input
        
        fn main () -> ::aquarium::internal::AnyhowResult<()> {
            let body = async {
                let proj = ::aquarium::internal::Project::load()?;
                let mut env = proj.env()?;
                #input_fn(&mut env).await;
                env.save_refs()?;
                Ok(())
            };
            #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
            {
                return ::aquarium::internal::tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(body);
            }
        }
    };
    output.into()
}
