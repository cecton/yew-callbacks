use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::quote;

#[proc_macro_derive(Callbacks, attributes(curry))]
pub fn main(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    derive_callbacks(&input).into()
}

fn derive_callbacks(input: &syn::DeriveInput) -> TokenStream {
    let enum_name = &input.ident;
    let vis = &input.vis;
    let e = match &input.data {
        syn::Data::Enum(e) => e,
        _ => abort_call_site!("`#[derive(Callbacks)]` only supports enums"),
    };

    let name = Ident::new(&format!("{enum_name}Callbacks"), Span::call_site());

    let field_names = e
        .variants
        .iter()
        .map(|variant| {
            Ident::new(
                &format!("callback_{}", variant.ident.to_string().to_snake_case()),
                Span::call_site(),
            )
        })
        .collect::<Vec<_>>();

    let inits = field_names
        .iter()
        .map(|field_name| {
            quote! {
                #field_name: Default::default(),
            }
        })
        .collect::<Vec<_>>();

    let curried_tys = e
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Unit => None,
            syn::Fields::Unnamed(syn::FieldsUnnamed {
                unnamed: fields, ..
            })
            | syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
                let tys = fields
                    .iter()
                    .filter(|field| is_curried(field))
                    .map(|field| &field.ty)
                    .collect::<Vec<_>>();

                if tys.is_empty() {
                    None
                } else {
                    Some(quote! {
                        (#(#tys),*)
                    })
                }
            }
        })
        .collect::<Vec<_>>();

    let tys = e
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Unit => {
                quote! {
                    ()
                }
            }
            syn::Fields::Unnamed(syn::FieldsUnnamed {
                unnamed: fields, ..
            })
            | syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
                let tys = fields
                    .iter()
                    .filter(|field| !is_curried(field))
                    .map(|field| &field.ty)
                    .collect::<Vec<_>>();

                quote! {
                    (#(#tys),*)
                }
            }
        })
        .collect::<Vec<_>>();

    let callbacks = field_names
        .iter()
        .zip(tys.iter())
        .zip(curried_tys.iter())
        .map(|((field_name, ty), curried_ty)| {
            if let Some(curried_ty) = curried_ty {
                quote! {
                    #field_name: ::std::cell::RefCell<
                        ::std::collections::HashMap<#curried_ty, ::yew::callback::Callback<#ty>>
                    >,
                }
            } else {
                quote! {
                    #field_name: ::std::cell::RefCell<Option<::yew::callback::Callback<#ty>>>,
                }
            }
        })
        .collect::<Vec<_>>();

    let constructors = e
        .variants
        .iter()
        .zip(tys.iter())
        .zip(field_names.iter())
        .zip(curried_tys.iter())
        .map(|(((variant, ty), field_name), curried_ty)| {
            let name = &variant.ident;
            let fn_name = Ident::new(&name.to_string().to_snake_case(), Span::call_site());

            match &variant.fields {
                syn::Fields::Unit => {
                    quote! {
                        fn #fn_name(&self) -> ::yew::callback::Callback<#ty> {
                            if self.#field_name.borrow().is_none() {
                                self.#field_name.replace(
                                    Some(self.link.callback(|_| #enum_name::#name))
                                );
                            }
                            self.#field_name.borrow().clone().unwrap()
                        }
                    }
                }
                syn::Fields::Unnamed(syn::FieldsUnnamed {
                    unnamed: fields, ..
                })
                | syn::Fields::Named(syn::FieldsNamed { named: fields, .. }) => {
                    let is_named = fields.iter().any(|field| field.ident.is_some());
                    let idents = fields
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            field.ident.clone().unwrap_or_else(|| {
                                Ident::new(&format!("arg_{i}"), Span::call_site())
                            })
                        })
                        .collect::<Vec<_>>();

                    if curried_ty.is_some() {
                        let args = fields
                            .iter()
                            .zip(idents.iter())
                            .filter_map(|(field, ident)| is_curried(field).then_some(ident))
                            .collect::<Vec<_>>();
                        let args_sig = fields
                            .iter()
                            .zip(idents.iter())
                            .filter(|(field, _)| is_curried(field))
                            .map(|(field, ident)| {
                                let ty = &field.ty;

                                quote! {
                                    #ident: #ty
                                }
                            })
                            .collect::<Vec<_>>();
                        let ins = fields
                            .iter()
                            .zip(idents.iter())
                            .filter_map(|(field, ident)| (!is_curried(field)).then_some(ident))
                            .collect::<Vec<_>>();
                        let keys = args
                            .iter()
                            .map(|arg| {
                                quote! {
                                    let #arg = #arg.clone();
                                }
                            })
                            .collect::<Vec<_>>();
                        let constructor = if is_named {
                            let cloned_args = fields
                                .iter()
                                .zip(idents.iter())
                                .map(|(field, ident)| {
                                    if is_curried(field) {
                                        quote! {
                                            #ident: #ident.clone()
                                        }
                                    } else {
                                        quote! {
                                            #ident
                                        }
                                    }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                #enum_name::#name { #(#cloned_args),* }
                            }
                        } else {
                            let cloned_args = fields
                                .iter()
                                .zip(idents.iter())
                                .map(|(field, ident)| {
                                    if is_curried(field) {
                                        quote! {
                                            #ident.clone()
                                        }
                                    } else {
                                        quote! {
                                            #ident
                                        }
                                    }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                #enum_name::#name(#(#cloned_args),*)
                            }
                        };

                        quote! {
                            #vis fn #fn_name(&self #(, #args_sig )* )
                                -> ::yew::callback::Callback<#ty>
                            {
                                self.#field_name
                                    .borrow_mut()
                                    .entry((#(#args),*))
                                    .or_insert_with_key(|(#(#args),*)| {
                                        #(#keys)*
                                        self.link.callback(move |(#(#ins),*)| #constructor)
                                    })
                                    .clone()
                            }
                        }
                    } else {
                        let constructor = if is_named {
                            quote! {
                                #enum_name::#name { #(#idents),* }
                            }
                        } else {
                            quote! {
                                #enum_name::#name(#(#idents),*)
                            }
                        };

                        quote! {
                            #vis fn #fn_name(&self) -> ::yew::callback::Callback<#ty> {
                                if self.#field_name.borrow().is_none() {
                                    self.#field_name.replace(Some(self
                                        .link
                                        .callback(|(#(#idents),*)| #constructor)
                                    ));
                                }
                                self.#field_name.borrow().clone().unwrap()
                            }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug)]
        #vis struct #name<C: ::yew::html::BaseComponent> {
            link: ::yew::html::Scope<C>,
            #(#callbacks)*
        }

        impl<C: ::yew::html::BaseComponent<Message = #enum_name>> #name<C> {
            #vis fn new(link: ::yew::html::Scope<C>) -> Self {
                Self {
                    link,
                    #(#inits)*
                }
            }

            #(#constructors)*
        }

        impl<C: ::yew::html::BaseComponent<Message = #enum_name>> From<::yew::html::Scope<C>>
            for #name<C>
        {
            fn from(link: ::yew::html::Scope<C>) -> Self {
                Self::new(link)
            }
        }

        impl<C: ::yew::html::BaseComponent<Message = #enum_name>> From<&::yew::html::Scope<C>>
            for #name<C>
        {
            fn from(link: &::yew::html::Scope<C>) -> Self {
                Self::new(link.to_owned())
            }
        }
    }
}

fn is_curried(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|x| x.path.get_ident().map(|x| x == "curry").unwrap_or(false))
}
