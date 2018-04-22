extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    let gen = impl_component(&ast);

    gen.into()
}

fn impl_component(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote!{
        impl ::unrust::engine::IntoComponentPtr for #name {
            fn into_component_ptr(self) -> ::std::sync::Arc<::unrust::engine::Component> {
                ::unrust::engine::Component::new(self)
            }
        }

        impl ::unrust::engine::ComponentBased for #name {            
        }
    }
}

#[proc_macro_derive(Actor)]
pub fn actor(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    let gen = impl_actor(&ast);

    gen.into()
}

fn impl_actor(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote!{
        impl ::unrust::engine::IntoComponentPtr for #name {
            fn into_component_ptr(self) -> ::std::sync::Arc<::unrust::engine::Component> {
                ::unrust::engine::Component::new(Box::new(self) as Box<::unrust::world::Actor> )
            }
        }        
    }
}