use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(AsJsonb)]
pub fn derive_as_jsonb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics diesel::deserialize::FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg>
            for #ident #ty_generics
            #where_clause
        {
            fn from_sql(bytes: diesel::pg::PgValue<'_>) -> diesel::deserialize::Result<Self> {
                let value =
                    <serde_json::Value as diesel::deserialize::FromSql<
                        diesel::sql_types::Jsonb,
                        diesel::pg::Pg,
                    >>::from_sql(bytes)?;

                Ok(serde_json::from_value(value)?)
            }
        }

        impl #impl_generics diesel::serialize::ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg>
            for #ident #ty_generics
            #where_clause
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                let value = serde_json::to_value(self)?;

                <serde_json::Value as diesel::serialize::ToSql<
                    diesel::sql_types::Jsonb,
                    diesel::pg::Pg,
                >>::to_sql(&value, &mut out.reborrow())
            }
        }
    }
    .into()
}
