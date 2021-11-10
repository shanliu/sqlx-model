use proc_macro::{TokenStream};
use quote::{quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Fields, Meta, NestedMeta, parse_macro_input};
use heck::{CamelCase, KebabCase, MixedCase, ShoutySnakeCase, SnakeCase};


fn get_sqlx_field_rename(attrs:&Vec<Attribute>)->Option<String>{
    for attr in attrs.iter(){
        let meta = attr
            .parse_meta()
            .map_err(|e| syn::Error::new_spanned(attr, e))
            .unwrap();
        match meta {
            Meta::List(list) => {
                for cattr in list.nested.iter(){
                    match cattr {
                        NestedMeta::Meta(Meta::NameValue(ref attr_ident)) =>{
                            let name=attr_ident.clone();
                            let name=name.path.get_ident().unwrap().to_string();
                            let name=name.as_str();
                            let ident=attr_ident.clone();
                            match name {
                                "rename"=>{
                                    let rename=match ident.lit {
                                        syn::Lit::Str(val)=>val,
                                        _=>unreachable!("rename be string"),
                                    }.value();
                                   return Some(rename);
                                },
                                _=>{}
                            }
                        }
                        _ => {},
                    }
                }
            }
            _ => {}
        }
    }
    None
}
fn change_sqlx_field_rename(change_type:&Option<String>,field_name:String)->String{
    if let Some(str)=change_type {
        match str.as_str() {
            "lowercase" => {
                return field_name.to_lowercase();
            },
            "snake_case" =>{
                return  field_name.to_snake_case();
            },
            "UPPERCASE" => {
                return   field_name.to_uppercase();
            },
            "SCREAMING_SNAKE_CASE" =>{
                return  field_name.to_shouty_snake_case();
            },
            "kebab-case"=> {
                return  field_name.to_kebab_case();
            },
            "camelCase" =>{
                return field_name.to_mixed_case();
            },
            "PascalCase" =>{
                return field_name.to_camel_case();
            },
            _ =>{},
        }
    }
    field_name
}


#[proc_macro_derive(SqlxModel, attributes(sqlx_model))]
// model 自动填充方法
pub fn derive_model(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let mut table_name=None;
    let mut rename_all=None;
    let mut table_pk=vec![];
    for attr in input.attrs.iter().filter(|e|{
        e.path.is_ident("sqlx_model")||e.path.is_ident("sqlx")
    })
    {
        let meta = attr
            .parse_meta()
            .map_err(|e| syn::Error::new_spanned(attr, e))
            .unwrap();
        match meta {
            Meta::List(list) => {
                for cattr in list.nested.iter(){
                    match cattr {
                        NestedMeta::Meta(Meta::NameValue(ref attr_ident)) =>{
                            let name=attr_ident.clone();
                            let name=name.path.get_ident().unwrap().to_string();
                            let name=name.as_str();
                            let ident=attr_ident.clone();
                            match name {
                                "table_name"=>{
                                    let val= match ident.lit {
                                        syn::Lit::Str(val)=>val,
                                        _=>unreachable!("table name must be string"),
                                    }.value();
                                    table_name=Some(val);
                                },
                                "table_pk"=>{
                                    let val= match ident.lit {
                                        syn::Lit::Str(val)=>val,
                                        _=>unreachable!("table pk field must be string"),
                                    }.value();
                                    table_pk.push(val);
                                },
                                "rename_all"=>{
                                    match ident.lit {
                                        syn::Lit::Str(val)=>{
                                            let str=&*val.value();
                                            rename_all =Some(str.to_owned());
                                        }
                                        _=>{},
                                    }
                                },
                                _=>{}
                            }
                        }
                        _ => {},
                    }
                }

            }
            _ => {}
        }
    }
    let table_name =table_name.unwrap_or_else(||{
        let mut name=struct_name.to_string();
        if name.clone().drain(0..5).collect::<String>()=="Model" {
            name=name.drain(5..).collect::<String>();
        }
        if name.clone().drain(name.len()-5..).collect::<String>()=="Model" {
            name=name.drain(0..name.len()-5).collect::<String>();
        }
        name.chars().enumerate().map(|(i,e)|{
            if i!=0 && e as u8>=65&&e as u8<=90 {
                format!("_{}",e.to_ascii_lowercase())
            }else{
                e.to_ascii_lowercase().to_string()
            }
        }).collect::<Vec<String>>().join("")
    });
    let expanded = match input.data {
        Data::Struct(DataStruct{ref fields,..}) => {
            if let Fields::Named(ref fields_name) = fields {
                let change_fields: Vec<_> = fields_name.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let str_field_name=match get_sqlx_field_rename(&field.attrs) {
                        Some(str)=>{str}
                        _=>{
                            change_sqlx_field_rename(&rename_all,field_name.to_string())
                        }
                    };
                    let field_type = field.ty.clone();
                    quote! {
                        #field_name[#str_field_name]:#field_type
                    }
                }).collect();
                let bind_fields: Vec<_> = fields_name.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let str_field_name=match get_sqlx_field_rename(&field.attrs) {
                        Some(str)=>{str}
                        _=>{
                            change_sqlx_field_rename(&rename_all,field_name.to_string())
                        }
                    };
                    quote! {
                        #field_name[#str_field_name]
                    }
                }).collect();
                let change_struct=quote::format_ident!("{}Ref",struct_name);
                let mut pk_fields=vec![];
                for field in fields_name.named.iter() {
                    let field_name = field.ident.as_ref().unwrap();
                    if table_pk.contains(&field_name.to_string()) {
                        let str_field_name=match get_sqlx_field_rename(&field.attrs) {
                            Some(str)=>{str}
                            _=>{
                                change_sqlx_field_rename(&rename_all,field_name.to_string())
                            }
                        };
                        pk_fields.push(quote! {
                            #field_name[#str_field_name]
                        });
                    }
                }
                if pk_fields.len()==0 {
                    if let Some(field)=fields_name.named.iter().next()  {
                        let field_name = field.ident.as_ref().unwrap();
                        let str_field_name=match get_sqlx_field_rename(&field.attrs) {
                            Some(str)=>{str}
                            _=>{
                                change_sqlx_field_rename(&rename_all,field_name.to_string())
                            }
                        };
                        pk_fields.push(quote! {
                            #field_name[#str_field_name]
                        });
                    }
                }
                let implemented_show = quote! {
                    sqlx_model::model_table_value_bind_define!(#struct_name,#table_name,{#(#bind_fields),*},{#(#pk_fields),*});
                    sqlx_model::model_table_ref_define!(#struct_name,#change_struct,{#(#change_fields),*});
                };
                implemented_show

            } else {
                panic!("sorry, may it's a complicated struct.");
            }
        }
        _ => panic!("sorry, Show is not implemented for union or enum type.")
    };
    expanded.into()
}
