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
    let gen = quote! {
        use crate::command::ProcessArg;

        impl ProcessArg for #name {
            fn process(&self, subcs: &clap::ArgMatches, v: Option<serde_json::Value>) {
                match subcs.subcommand() {
                    Some(("custom", args)) => {
                        self.set_mirror_by_args(args);
                    }
                    Some(("select", _)) => {
                        self.select();
                    }
                    Some(("default", _)) => {
                        if let Some(v) = v {
                            self.set_mirror_by_value(v);
                        }
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
