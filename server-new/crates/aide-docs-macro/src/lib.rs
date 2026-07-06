use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, ItemFn, LitStr, Result, Token, Type, parse::Parse, parse::ParseStream,
    parse_macro_input,
};

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

struct ApiRoutes {
    state: Type,
    tag: Expr,
    routes: Vec<ApiRoute>,
}

struct ApiRoute {
    methods: Vec<RouteMethod>,
    path: LitStr,
    handler: Ident,
    summary: LitStr,
    description: Option<LitStr>,
}

struct RouteMethod {
    ident: Ident,
}

impl RouteMethod {
    fn route_fn(&self) -> Result<Ident> {
        let method = self.ident.to_string();
        let fn_name = match method.as_str() {
            "GET" => "get_with",
            "POST" => "post_with",
            "DELETE" => "delete_with",
            _ => {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "expected one of GET, POST, DELETE",
                ));
            }
        };

        Ok(format_ident!("{fn_name}"))
    }
}

impl Parse for ApiRoutes {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        parse_label(input, "state")?;
        input.parse::<Token![:]>()?;
        let state = input.parse()?;
        input.parse::<Token![,]>()?;

        parse_label(input, "tag")?;
        input.parse::<Token![:]>()?;
        let tag = input.parse()?;
        input.parse::<Token![,]>()?;

        let mut routes = Vec::new();
        while !input.is_empty() {
            routes.push(input.parse()?);
        }

        Ok(Self { state, tag, routes })
    }
}

impl Parse for ApiRoute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut methods = vec![RouteMethod {
            ident: input.parse()?,
        }];

        while input.peek(Token![,]) {
            let fork = input.fork();
            fork.parse::<Token![,]>()?;
            if fork.peek(Ident) {
                input.parse::<Token![,]>()?;
                methods.push(RouteMethod {
                    ident: input.parse()?,
                });
            } else {
                break;
            }
        }

        let path = input.parse()?;
        input.parse::<Token![=>]>()?;
        let handler = input.parse()?;
        input.parse::<Token![,]>()?;
        let summary = input.parse()?;

        let description = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![;]>()?;

        Ok(Self {
            methods,
            path,
            handler,
            summary,
            description,
        })
    }
}

fn parse_label(input: ParseStream<'_>, expected: &str) -> Result<()> {
    let label: Ident = input.parse()?;
    if label == expected {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            label,
            format!("expected `{expected}`"),
        ))
    }
}

/// Generate a `routes()` function and the matching `<handler>_docs` functions.
///
/// # Syntax
/// ```ignore
/// api_routes! {
///     state: AppState,
///     tag: ApiTag::Auth,
///     GET "/user" => get_user, "Get user", "Get the current user";
///     GET, POST "/logout" => logout, "Logout";
/// }
/// ```
#[proc_macro]
pub fn api_routes(input: TokenStream) -> TokenStream {
    let api_routes = parse_macro_input!(input as ApiRoutes);
    let state = api_routes.state;
    let tag = api_routes.tag;

    let docs_functions = api_routes.routes.iter().map(|route| {
        let docs_name = format_ident!("{}_docs", route.handler);
        let handler_name = route.handler.to_string();
        let summary = &route.summary;
        let description = route.description.as_ref().map(|description| {
            quote! {
                .description(#description)
            }
        });

        quote! {
            fn #docs_name(
                op: ::aide::transform::TransformOperation,
            ) -> ::aide::transform::TransformOperation {
                op.id(#handler_name)
                    .summary(#summary)
                    #description
            }
        }
    });

    let route_calls = api_routes.routes.iter().map(|route| {
        let path = &route.path;
        let handler = &route.handler;
        let docs_name = format_ident!("{}_docs", route.handler);

        let mut methods = route
            .methods
            .iter()
            .map(RouteMethod::route_fn)
            .collect::<Result<Vec<_>>>()?;
        let first_method = methods.remove(0);

        Ok(quote! {
            .api_route(
                #path,
                ::aide::axum::routing::#first_method(#handler, #docs_name)
                    #(.#methods(#handler, #docs_name))*
            )
        })
    });

    let route_calls = match route_calls.collect::<Result<Vec<_>>>() {
        Ok(route_calls) => route_calls,
        Err(error) => return error.into_compile_error().into(),
    };

    quote! {
        #(#docs_functions)*

        pub fn routes() -> ::aide::axum::ApiRouter<#state> {
            ::aide::axum::ApiRouter::new()
                #(#route_calls)*
                .with_path_items(|op| op.tag(#tag.into()))
        }
    }
    .into()
}
