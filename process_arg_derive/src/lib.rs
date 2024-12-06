use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(ProcessArg)]
pub fn process_macro_derive(input: TokenStream) -> TokenStream {
    // 基于 input 构建 AST 语法树
    let ast: DeriveInput = syn::parse(input).unwrap();
    // 构建特征实现代码
    impl_process_macro(&ast)
}

fn impl_process_macro(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let gen = quote! {
        impl  ProcessArg<#generics> for #name {
            fn process(&self, subcs: &clap::ArgMatches) {
                match subcs.subcommand() {
                    Some(("config", args)) => {
                        self.set_mirror(args);
                    }
                    Some(("reset", _)) => {
                        self.reset_mirrors();
                    }
                    Some(("get", _)) => {
                        let mirror = self.current_mirror();
                        println!("{:#?}", mirror)
                    }
                    Some((_, _)) | None => {}
                }
            }
        }
    };
    gen.into()
}
