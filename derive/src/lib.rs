use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    parse2, parse_macro_input, parse_quote, Attribute, DeriveInput, Field, GenericParam, Generics,
    Ident, Meta, Type, Visibility,
};

#[proc_macro_derive(ShaderData, attributes(shader))]
pub fn derive_shader_data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let vis = input.vis;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_data = match input.data {
        syn::Data::Struct(struct_data) => struct_data,
        _ => {
            return quote_spanned! {
                Span::call_site() =>
                compile_error!("ShaderData can only be derived for struct types");
            }
            .into();
        }
    };

    let shader_vars_name = Ident::new(&format!("{}__ShaderVars", name), name.span());
    let (shader_vars, shader_collect, shader_write) = match struct_data.fields {
        syn::Fields::Named(fields) => {
            let fields = ShaderField::extract(fields.named.into_iter());

            let shader_vars_fields = fields
                .iter()
                .map(|x| {
                    let vis = &x.vis;
                    let ident = x.ident.as_ref().unwrap();
                    let ty = x.ty_encoder.as_ref().unwrap_or(&x.ty);
                    quote! { #vis #ident: <#ty as picodraw::ShaderData>::ShaderVars }
                })
                .collect::<Vec<_>>();

            let shader_collect_fields = fields
                .iter()
                .map(|x| {
                    let ident = x.ident.as_ref().unwrap();
                    let ident_str = ident.to_string();
                    let ty = x.ty_encoder.as_ref().unwrap_or(&x.ty);
                    quote! { #ident: <#ty as picodraw::ShaderData>::shader_vars(&mut picodraw::prefix_vars(vars, #ident_str)) }
                })
                .collect::<Vec<_>>();

            let shader_write_fields = fields
                .iter()
                .map(|x| {
                    let ident = x.ident.as_ref().unwrap();
                    let ident_str = ident.to_string();
                    let ty = x.ty_encoder.as_ref().unwrap_or(&x.ty);

                    if x.ty_encoder.is_some() {
                        quote! { <#ty as picodraw::ShaderData>::write(&self.#ident.into(), &mut picodraw::prefix_writer(writer, #ident_str)); }
                    } else {
                        quote! { <#ty as picodraw::ShaderData>::write(&self.#ident, &mut picodraw::prefix_writer(writer, #ident_str)); }
                    }
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
            let fields = ShaderField::extract(fields.unnamed.into_iter());

            let shader_vars_fields = fields
                .iter()
                .map(|x| {
                    let vis = &x.vis;
                    let ty = x.ty_encoder.as_ref().unwrap_or(&x.ty);
                    quote! { #vis <#ty as picodraw::ShaderData>::ShaderVars }
                })
                .collect::<Vec<_>>();

            let shader_collect_fields = fields
                .iter()
                .map(|x| {
                    let id = x.index;
                    let ty = x.ty_encoder.as_ref().unwrap_or(&x.ty);
                    quote! { <#ty as picodraw::ShaderData>::shader_vars(&mut picodraw::prefix_vars(vars, stringify!(#id))) }
                })
                .collect::<Vec<_>>();

            let shader_write_fields = fields
                .iter()
                .map(|x| {
                    let id = x.index;
                    let ty = x.ty_encoder.as_ref().unwrap_or(&x.ty);
                    if x.ty_encoder.is_some() {
                        quote! { <#ty as picodraw::ShaderData>::write(&self.#id.into(), &mut picodraw::prefix_writer(writer, stringify!(#id))); }
                    } else {
                        quote! { <#ty as picodraw::ShaderData>::write(&self.#id, &mut picodraw::prefix_writer(writer, stringify!(#id))); }
                    }
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
        #[doc(hidden)]
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

struct ShaderField {
    vis: Visibility,
    index: usize,
    ident: Option<Ident>,
    ty: Type,
    ty_encoder: Option<Type>,
}

enum ShaderAttribute {
    Ignore,
    EncoderType(Type),
}

impl ShaderAttribute {
    fn extract(attrs: Vec<Attribute>) -> Option<Self> {
        let mut shader_attr = None;
        for attr in attrs {
            if let Meta::List(meta) = attr.meta {
                if meta.path.is_ident("shader") {
                    if meta.tokens.to_string() == "ignore" {
                        shader_attr = Some(ShaderAttribute::Ignore);
                    } else {
                        shader_attr = Some(Self::EncoderType(parse2(meta.tokens).expect(
                            "invalid shader attribute structure, should be #[shader(Type)]",
                        )));
                    }

                    break;
                }
            }
        }
        shader_attr
    }
}

impl ShaderField {
    fn extract(fields: impl IntoIterator<Item = Field>) -> Vec<ShaderField> {
        fields
            .into_iter()
            .enumerate()
            .filter_map(
                |(index, field)| match ShaderAttribute::extract(field.attrs) {
                    Some(ShaderAttribute::Ignore) => None,
                    Some(ShaderAttribute::EncoderType(ty_encoder)) => Some(ShaderField {
                        vis: field.vis,
                        index,
                        ident: field.ident,
                        ty: field.ty,
                        ty_encoder: Some(ty_encoder),
                    }),
                    None => Some(ShaderField {
                        vis: field.vis,
                        index,
                        ident: field.ident,
                        ty: field.ty,
                        ty_encoder: None,
                    }),
                },
            )
            .collect()
    }
}
