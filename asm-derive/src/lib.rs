use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FixIntermediate)]
pub fn derive_fix_intermediate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract enum variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("FixIntermediate can only be derived for enums"),
    };

    // Generate match arms for each variant
    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Check if this variant has named fields
        if let Fields::Named(fields_named) = &variant.fields {
            // Check if it has src, dest, and size fields
            let has_src = fields_named.named.iter().any(|f| {
                f.ident.as_ref().map_or(false, |ident| ident == "src")
            });

            let has_dest = fields_named.named.iter().any(|f| {
                f.ident.as_ref().map_or(false, |ident| ident == "dest")
            });

            let has_size = fields_named.named.iter().any(|f| {
                f.ident.as_ref().map_or(false, |ident| ident == "size")
            });

            if has_src && has_dest && has_size {
                // Check for operator field (for Binary variant)
                let has_operator = fields_named.named.iter().any(|f| {
                    f.ident.as_ref().map_or(false, |ident| ident == "operator")
                });

                if has_operator {
                    // Pattern for Binary-like variants (with operator field)
                    quote! {
                        Self::#variant_name { operator, size, src, dest, .. } => {
                            if matches!(src.as_ref(), Operand::Register(Pseudoregister::Pseudoregister(_, _) | Pseudoregister::Data(_, _)) | Operand::MemoryReference(_, _, _)) && matches!(dest.as_ref(), Pseudoregister::Pseudoregister(_, _) | Pseudoregister::Data(_, _)) {
                                let r10 = std::rc::Rc::from(Pseudoregister::Register(
                                    Reg::R10,
                                    if *size == 4 { Type::Int } else { Type::Long },
                                ));
                                out.push_back(Self::Mov {
                                    size: *size,
                                    src: src.clone(),
                                    dest: r10.clone(),
                                });
                                out.push_back(Self::#variant_name {
                                    operator: *operator,
                                    size: *size,
                                    src: std::rc::Rc::from(Operand::Register(r10.as_ref().clone())),
                                    dest: dest.clone(),
                                });
                            } else {
                                out.push_back(self.clone());
                            }
                        },
                    }
                } else {
                    // Pattern for Mov-like variants (without operator field)
                    quote! {
                        Self::#variant_name { size, src, dest, .. } => {
                            if matches!(src.as_ref(), Operand::Register(Pseudoregister::Pseudoregister(_, _) | Pseudoregister::Data(_, _)) | Operand::MemoryReference(_, _, _)) && matches!(dest.as_ref(), Pseudoregister::Pseudoregister(_, _) | Pseudoregister::Data(_, _)) {
                                let r10 = std::rc::Rc::from(Pseudoregister::Register(
                                    Reg::R10,
                                    if *size == 4 { Type::Int } else { Type::Long },
                                ));
                                out.push_back(Self::#variant_name {
                                    size: *size,
                                    src: src.clone(),
                                    dest: r10.clone(),
                                });
                                out.push_back(Self::#variant_name {
                                    size: *size,
                                    src: std::rc::Rc::from(Operand::Register(r10.as_ref().clone())),
                                    dest: dest.clone(),
                                });
                            } else {
                                out.push_back(self.clone());
                            }
                        },
                    }
                }
            } else {
                // Other variants with named fields - do nothing
                quote! {
                    Self::#variant_name { .. } => out.push_back(self.clone()),
                }
            }
        } else if let Fields::Unnamed(_) = &variant.fields {
            // Tuple variants - do nothing
            quote! {
                Self::#variant_name(..) => out.push_back(self.clone()),
            }
        } else {
            // Unit variants - do nothing
            quote! {
                Self::#variant_name => out.push_back(self.clone()),
            }
        }
    });

    // Generate implementation
    let expanded = quote! {
        impl #name {
            fn fix_intermediate(&self, out: &mut std::collections::VecDeque<Self>) {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
