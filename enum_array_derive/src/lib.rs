use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(EnumDiscriminant)]
pub fn derive_enum_discriminant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new_spanned(&input, "EnumDiscriminant can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let mut discriminant_arms = Vec::new();
    let variant_count = data_enum.variants.len();

    for (index, variant) in data_enum.variants.iter().enumerate() {
        let variant_name = &variant.ident;
        let pattern = match &variant.fields {
            Fields::Named(_) => quote! { Self::#variant_name { .. } },
            Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
            Fields::Unit => quote! { Self::#variant_name },
        };

        discriminant_arms.push(quote! {
            #pattern => #index
        });
    }

    let expanded = quote! {
        impl enum_array_trait::EnumDiscriminant for #name {
            const COUNT: usize = #variant_count;

            fn discriminant(&self) -> usize {
                match self {
                    #(#discriminant_arms,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
