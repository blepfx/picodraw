use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, parse_quote, DeriveInput, GenericParam, Generics, Ident};

#[proc_macro_derive(ShaderData)]
pub fn derive_shader_data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let vis = input.vis;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let str = match input.data {
        syn::Data::Struct(s) => s,
        _ => {
            return quote_spanned! {
                Span::call_site() =>
                compile_error!("ShaderData can only be derived for struct types");
            }
            .into();
        }
    };

    let shader_vars_name = Ident::new(&format!("{}__ShaderVars", name), name.span());

    let (shader_vars, shader_collect, shader_write) = match str.fields {
        syn::Fields::Named(fields) => {
            let shader_vars_fields = fields
                .named
                .iter()
                .map(|x| {
                    let vis = &x.vis;
                    let ident = x.ident.as_ref().unwrap();
                    let ty = &x.ty;
                    quote! { #vis #ident: <#ty as picodraw::ShaderData>::ShaderVars }
                })
                .collect::<Vec<_>>();

            let shader_collect_fields = fields
                .named
                .iter()
                .map(|x| {
                    let ident = x.ident.as_ref().unwrap();
                    let ident_str = ident.to_string();
                    let ty = &x.ty;
                    quote! { #ident: <#ty as picodraw::ShaderData>::shader_vars(&mut picodraw::prefix_vars(vars, #ident_str)) }
                })
                .collect::<Vec<_>>();

            let shader_write_fields = fields
                .named
                .iter()
                .map(|x| {
                    let ident = x.ident.as_ref().unwrap();
                    let ident_str = ident.to_string();
                    let ty = &x.ty;
                    quote! { <#ty as picodraw::ShaderData>::write(&self.#ident, &mut picodraw::prefix_writer(writer, #ident_str)); }
                })
                .collect::<Vec<_>>();

            (
                quote! {
                    #vis struct #shader_vars_name #generics {
                        #(#shader_vars_fields),*
                    }
                },
                quote! {
                    #shader_vars_name {
                        #(#shader_collect_fields),*
                    }
                },
                quote! {
                    #(#shader_write_fields)*
                },
            )
        }

        syn::Fields::Unnamed(fields) => {
            let shader_vars_fields = fields
                .unnamed
                .iter()
                .map(|x| {
                    let vis = &x.vis;
                    let ty = &x.ty;
                    quote! { #vis <#ty as picodraw::ShaderData>::ShaderVars }
                })
                .collect::<Vec<_>>();

            let shader_collect_fields = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(id, x)| {
                    let ty = &x.ty;
                    quote! { <#ty as picodraw::ShaderData>::shader_vars(&mut picodraw::prefix_vars(vars, stringify!(#id))) }
                })
                .collect::<Vec<_>>();

            let shader_write_fields = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(id, x)| {
                    let ty = &x.ty;
                    quote! { <#ty as picodraw::ShaderData>::write(&self.#id, &mut picodraw::prefix_writer(writer, stringify!(#id))); }
                })
                .collect::<Vec<_>>();

            (
                quote! {
                    #vis struct #shader_vars_name #generics (#(#shader_vars_fields),*);
                },
                quote! {
                    #shader_vars_name (#(#shader_collect_fields),*)
                },
                quote! {
                    #(#shader_write_fields)*
                },
            )
        }

        syn::Fields::Unit => (
            quote! { #vis struct #shader_vars_name; },
            quote! { let _ = vars; () },
            quote! { let _ = writer; },
        ),
    };

    quote! {
        #[allow(non_camel_case_types)]
        #shader_vars

        impl #impl_generics picodraw::ShaderData for #name #ty_generics #where_clause {
            type ShaderVars = #shader_vars_name #ty_generics;
            fn shader_vars(vars: &mut dyn picodraw::ShaderVars) -> Self::ShaderVars {
                #shader_collect
            }
            fn write(&self, writer: &mut dyn picodraw::ShaderDataWriter) {
                #shader_write
            }
        }
    }
    .into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(::picodraw::ShaderData));
        }
    }
    generics
}
