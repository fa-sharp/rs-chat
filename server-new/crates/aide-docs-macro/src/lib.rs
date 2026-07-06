use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr, Result, Token, parse::Parse, parse::ParseStream, parse_macro_input};

struct DocsArgs {
    summary: LitStr,
    description: Option<LitStr>,
}

impl Parse for DocsArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let summary = input.parse()?;
        let description = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            summary,
            description,
        })
    }
}

/// Convenience macro for generating API docs with aide. Generates a function
/// called `<handler_name>_docs` that can be passed as the transform function
/// to `get_with`, `post_with`, etc.
///
/// # Syntax
/// `#[docs("<summary>", "<description>"]`
#[proc_macro_attribute]
pub fn docs(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as DocsArgs);
    let handler = parse_macro_input!(input as ItemFn);
    let handler_name = &handler.sig.ident;
    let docs_name = format_ident!("{}_docs", handler_name);
    let operation_id = handler_name.to_string();
    let summary = args.summary;

    let description = args.description.map(|description| {
        quote! {
            .description(#description)
        }
    });

    quote! {
        fn #docs_name(
            op: ::aide::transform::TransformOperation,
        ) -> ::aide::transform::TransformOperation {
            op.id(#operation_id)
                .summary(#summary)
                #description
        }

        #handler
    }
    .into()
}
