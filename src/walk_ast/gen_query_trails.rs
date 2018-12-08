use super::{ident, AddToOutput, Output};
use graphql_parser::schema::Definition::*;
use graphql_parser::schema::TypeDefinition::*;
use graphql_parser::schema::*;
use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_query_trails(doc: &Document, out: &mut Output) {
    gen_query_trail(out);

    for def in &doc.definitions {
        match def {
            TypeDefinition(type_def) => match type_def {
                Object(obj) => gen_field_walk_methods(obj, out),
                _ => {}
            },
            _ => {}
        }
    }
}

fn gen_query_trail(out: &mut Output) {
    (quote! {
        /// A wrapper around a `juniper::LookAheadSelection` with methods for each possible child.
        ///
        /// Generated by `juniper-from-schema`.
        pub struct QueryTrail<'a, T> {
            look_ahead: Option<&'a juniper::LookAheadSelection<'a>>,
            phantom: std::marker::PhantomData<T>,
        }

        impl<'a, T> QueryTrail<'a, T> {
            fn is_present(&self) -> bool {
                self.look_ahead.is_some()
            }
        }

        trait MakeTraversal<'a> {
            fn make_query_trail<T>(&'a self) -> QueryTrail<'a, T>;
        }

        impl<'a> MakeTraversal<'a> for juniper::LookAheadSelection<'a> {
            fn make_query_trail<T>(&'a self) -> QueryTrail<'a, T> {
                QueryTrail {
                    look_ahead: Some(self),
                    phantom: std::marker::PhantomData,
                }
            }
        }
    })
    .add_to(out);
}

fn gen_field_walk_methods(obj: &ObjectType, out: &mut Output) {
    let name = ident(&obj.name);
    let methods = obj.fields.iter().map(|field| gen_field_walk_method(field));

    (quote! {
        impl<'a> QueryTrail<'a, #name> {
            #(#methods)*
        }
    })
    .add_to(out)
}

fn gen_field_walk_method(field: &Field) -> TokenStream {
    let name = ident(&field.name.to_snake_case());
    let field_type = quote! { Club };
    let string_name = &field.name;

    quote! {
        /// Walk the trail into a field.
        ///
        /// Generated by `juniper-from-schema`.
        pub fn #name(&self) -> QueryTrail<'a, #field_type> {
            use juniper::LookAheadMethods;

            let child = self.look_ahead.and_then(|la| la.select_child(#string_name));

            self::QueryTrail {
                look_ahead: child,
                phantom: std::marker::PhantomData,
            }
        }
    }
}
