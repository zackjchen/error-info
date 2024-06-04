use darling::{
    ast::{Data, Fields},
    util, FromDeriveInput, FromVariant,
};
use quote::quote;
use syn::DeriveInput;

#[allow(unused)]
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ErrorData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<EnumVariants, ()>,
    app_type: syn::Type,
    prefix: String,
}

#[allow(unused)]
#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumVariants {
    ident: syn::Ident,
    fields: Fields<util::Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

pub(crate) fn process_error_info(input: DeriveInput) -> proc_macro2::TokenStream {
    // 模式匹配
    let ErrorData {
        ident: name, // MyError
        generics,
        data: Data::Enum(fields),
        app_type, // http::StatusCode
        prefix,   // 01
    } = ErrorData::from_derive_input(&input).unwrap()
    else {
        panic!("Only enum is supported")
    };

    let code = fields
        .iter()
        .map(|v| {
            let EnumVariants {
                ident,
                code,
                app_code,
                client_msg,
                fields,
            } = v;
            let code = format!("{}{}", prefix, code);
            let varint_code = match fields.style {
                darling::ast::Style::Tuple => quote! { #name::#ident{..}},
                darling::ast::Style::Struct => quote! { #name::#ident(_)},
                darling::ast::Style::Unit => quote! { #name::#ident},
            };
            quote! {
                #varint_code => ErrorInfo::try_new(#app_code, #code, #client_msg, self),
            }
        })
        .collect::<Vec<_>>();
    quote! {
        use error_code::{ErrorInfo, ToErrorInfo as _};
        impl #generics ToErrorInfo for #name #generics {
            type T = #app_type;
            fn to_error_info(&self) -> Result<ErrorInfo<Self::T>, <Self::T as std::str::FromStr>::Err> {
                match self {
                    #(#code)*
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    #[test]
    fn test_darling_data_struct() -> Result<()> {
        let input = r#"
        #[derive(ToErrorInfo,thiserror::Error)]
        #[error_info(app_type = "http::StatusCode", prefix = "01")]
        pub enum MyError {
            #[error("Bad Request:{0}")]
            #[error_info(code = "BR",app_code = "400", client_msg = "Bad Request")]
            BadRequest(String),
            #[error("Unauthorized:{0}")]
            #[error_info(code = "U",app_code = "401", client_msg = "Unauthorized")]
            Unauthorized(String),
            #[error("Forbidden")]
            #[error_info(code = "FB", app_code = "403", client_msg = "Forbidden")]
            Forbidden,
            #[error("Not Found")]
            #[error_info(code = "NF", app_code = "404", client_msg = "Not Found")]
            NotFound,
            #[error("Internal Server Error")]
            #[error_info(code = "ISE",app_code = "500", client_msg = "Internal Server Error")]
            InternalServerError(#[from] anyhow::Error),
        }
        "#;
        let input = syn::parse_str(input).unwrap();
        let ast = ErrorData::from_derive_input(&input).unwrap();
        // println!("{:#?}",ast);
        assert_eq!(ast.ident.to_string(), "MyError");
        assert_eq!(ast.prefix, "01");
        let code = process_error_info(input);
        println!("{:#?}", code);
        Ok(())
    }
}
