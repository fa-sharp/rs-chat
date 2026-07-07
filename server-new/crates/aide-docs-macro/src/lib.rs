use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, ItemFn, LitInt, LitStr, Result, Token, Type, braced, parse::Parse,
    parse::ParseStream, parse_macro_input,
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

/// Convenience macro for generating API docs for the route handler. Generates a function
/// called `<handler_name>_docs` that can be passed as the transform function
/// to aide's `get_with`, `post_with`, etc.
///
/// # Syntax
/// `#[handler_docs("<summary>" (, "<description>")]`
#[proc_macro_attribute]
pub fn handler_docs(args: TokenStream, input: TokenStream) -> TokenStream {
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
    tag: Option<Expr>,
    routes: Vec<ApiRoute>,
}

struct ApiRoute {
    methods: Vec<RouteMethod>,
    path: LitStr,
    handler: Ident,
    summary: Option<LitStr>,
    description: Option<LitStr>,
    responses: Vec<RouteResponse>,
}

struct RouteResponse {
    status: LitInt,
    ty: Type,
}

struct RouteMethod {
    ident: Ident,
}

impl RouteMethod {
    fn name(&self) -> String {
        self.ident.to_string()
    }

    fn operation_prefix(&self) -> String {
        self.name().to_lowercase()
    }

    fn route_fn(&self) -> Result<Ident> {
        let fn_name = match self.name().as_str() {
            "GET" => "get_with",
            "POST" => "post_with",
            "PATCH" => "patch_with",
            "DELETE" => "delete_with",
            _ => {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "expected one of GET, POST, PATCH, DELETE",
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

        let tag = if next_label_is(input, "tag") {
            parse_label(input, "tag")?;
            input.parse::<Token![:]>()?;
            let tag = input.parse()?;
            input.parse::<Token![,]>()?;
            Some(tag)
        } else {
            None
        };

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

        let summary = if input.peek(LitStr) {
            Some(input.parse()?)
        } else {
            None
        };

        let options = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.peek(syn::token::Brace) {
                return Err(input.error("expected route options block after summary comma"));
            }
            input.parse()?
        } else if input.peek(syn::token::Brace) {
            input.parse()?
        } else {
            RouteOptions::default()
        };

        input.parse::<Token![;]>()?;

        if summary.is_none() && options.is_empty() {
            return Err(input.error("expected summary string or route options block"));
        }

        Ok(Self {
            methods,
            path,
            handler,
            summary,
            description: options.description,
            responses: options.responses,
        })
    }
}

#[derive(Default)]
struct RouteOptions {
    description: Option<LitStr>,
    responses: Vec<RouteResponse>,
}

impl RouteOptions {
    fn is_empty(&self) -> bool {
        self.description.is_none() && self.responses.is_empty()
    }
}

impl Parse for RouteOptions {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut options = RouteOptions::default();
        while !content.is_empty() {
            let label: Ident = content.parse()?;
            content.parse::<Token![:]>()?;

            match label.to_string().as_str() {
                "description" => {
                    if options.description.is_some() {
                        return Err(syn::Error::new_spanned(
                            label,
                            "`description` can only be provided once",
                        ));
                    }

                    options.description = Some(content.parse()?);
                }
                "responses" => {
                    if !options.responses.is_empty() {
                        return Err(syn::Error::new_spanned(
                            label,
                            "`responses` can only be provided once",
                        ));
                    }

                    let responses;
                    braced!(responses in content);

                    while !responses.is_empty() {
                        options.responses.push(responses.parse()?);
                        if responses.peek(Token![,]) {
                            responses.parse::<Token![,]>()?;
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        label,
                        "expected `description` or `responses`",
                    ));
                }
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(options)
    }
}

impl Parse for RouteResponse {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let status = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;

        Ok(Self { status, ty })
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

fn next_label_is(input: ParseStream<'_>, expected: &str) -> bool {
    let fork = input.fork();
    fork.parse::<Ident>().is_ok_and(|label| label == expected) && fork.peek(Token![:])
}

/// Generate a `routes()` function with attached API docs
///
/// # Syntax
/// ```ignore
/// api_routes! {
///     state: AppState,
///     tag: ApiTag::Auth, // optional
///     GET "/user" => get_user, "Get user", {
///         description: "Get the current user"
///     };
///     GET, POST "/logout" => logout, "Logout", {
///         responses: { 204: () }
///     };
///     POST "/sessions" => create_session, {
///         description: "Create a new session",
///         responses: { 201: Session }
///     };
/// }
/// ```
#[proc_macro]
pub fn api_routes(input: TokenStream) -> TokenStream {
    let api_routes = parse_macro_input!(input as ApiRoutes);
    let state = api_routes.state;

    let docs_functions = api_routes.routes.iter().flat_map(|route| {
        let summary = route.summary.as_ref().map(|summary| {
            quote! {
                .summary(#summary)
            }
        });
        let description = route.description.as_ref().map(|description| {
            quote! {
                .description(#description)
            }
        });
        let responses = route
            .responses
            .iter()
            .map(|response| {
                let status = &response.status;
                let ty = &response.ty;

                quote! {
                    .response::<#status, #ty>()
                }
            })
            .collect::<Vec<_>>();
        let multiple_methods = route.methods.len() > 1;

        route.methods.iter().map(move |method| {
            let docs_name = docs_name(route, method, multiple_methods);
            let operation_id = operation_id(route, method, multiple_methods);

            quote! {
            fn #docs_name(
                op: ::aide::transform::TransformOperation,
            ) -> ::aide::transform::TransformOperation {
                op.id(#operation_id)
                    #summary
                    #description
                    #(#responses)*
            }
            }
        })
    });

    let route_calls = api_routes.routes.iter().map(|route| {
        let path = &route.path;
        let handler = &route.handler;
        let multiple_methods = route.methods.len() > 1;

        let mut methods = route
            .methods
            .iter()
            .map(|method| {
                let route_fn = method.route_fn()?;
                let docs_name = docs_name(route, method, multiple_methods);

                Ok((route_fn, docs_name))
            })
            .collect::<Result<Vec<_>>>()?;
        let (first_method, first_docs_name) = methods.remove(0);

        let additional_methods = methods.into_iter().map(|(method, docs_name)| {
            quote! {
                .#method(#handler, #docs_name)
            }
        });

        Ok(quote! {
            .api_route(
                #path,
                ::aide::axum::routing::#first_method(#handler, #first_docs_name)
                    #(#additional_methods)*
            )
        })
    });

    let route_calls = match route_calls.collect::<Result<Vec<_>>>() {
        Ok(route_calls) => route_calls,
        Err(error) => return error.into_compile_error().into(),
    };

    let tag = api_routes.tag.map(|tag| {
        quote! {
            .with_path_items(|op| op.tag(#tag.into()))
        }
    });

    quote! {
        #(#docs_functions)*

        pub fn routes() -> ::aide::axum::ApiRouter<#state> {
            ::aide::axum::ApiRouter::new()
                #(#route_calls)*
                #tag
        }
    }
    .into()
}

fn docs_name(route: &ApiRoute, method: &RouteMethod, multiple_methods: bool) -> Ident {
    if multiple_methods {
        let method = method.operation_prefix();
        format_ident!("{}_{}_docs", method, route.handler)
    } else {
        format_ident!("{}_docs", route.handler)
    }
}

fn operation_id(route: &ApiRoute, method: &RouteMethod, multiple_methods: bool) -> String {
    if multiple_methods {
        format!("{}_{}", method.operation_prefix(), route.handler)
    } else {
        route.handler.to_string()
    }
}
