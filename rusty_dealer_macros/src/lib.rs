use proc_macro::TokenStream;

#[proc_macro_derive(SuitIter, attributes(spades, hearts, diamonds, clubs))]
pub fn suititer_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_persuit_macro(ast)
}

fn impl_persuit_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = ast.ident;
    let mut to_iter_over: Vec<_> = match ast.data {
        syn::Data::Struct(ref s) => s
            .fields
            .iter()
            .filter(|field| !field.attrs.is_empty())
            .collect(),
        _ => panic!("use only on structs"),
    };
    to_iter_over.sort_by_key(|a| field_to_index(a));
    let to_iter_over = to_iter_over
        .into_iter()
        .map(|field| field.ident.as_ref().unwrap());
    let data = quote::quote! {
        impl SuitIter for #name {
            fn iter() -> SuitIter {
                SuitIter {
                    suitwise: [#(#to_iter_over),*].iter()
                }
            }
        }
    };
    data.into()
}

fn field_to_index(field: &syn::Field) -> usize {
    for attr in &field.attrs {
        if attr.path().is_ident("spades") {
            return 0;
        } else if attr.path().is_ident("hearts") {
            return 1;
        } else if attr.path().is_ident("diamonds") {
            return 2;
        } else if attr.path().is_ident("clubs") {
            return 3;
        }
    }
    panic!("unknown attr")
}

#[proc_macro_derive(RawDDS)]
pub fn rawdds_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_rawdds_macro(ast)
}

fn impl_rawdds_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = ast.ident;
    let data = match ast.data {
        syn::Data::Struct(data) => data,
        _ => unimplemented!(),
    };
    let field: syn::Field = data.fields.into_iter().next().unwrap();

    let ty = field.ty;
    let field_ident = field.ident.unwrap();
    quote::quote! {
        impl RawDDS for #name {
            type Raw = #ty;

            fn get_raw(&self) -> Self::Raw {
                self.#field_ident
            }
        }
    }
    .into()
}