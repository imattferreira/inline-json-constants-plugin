/**
 * TODO
 * - add possibility to customize constants dir entrypoint
 * - debug
 * - script not working
 * - TEST THIS SHIT!
 */
mod utils;
use swc_core::ecma::visit::*;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use swc_ecma_ast::{CallExpr, Callee, Expr, Program};
use swc_ecma_visit::Fold;
use tracing::{event, Level};
use utils::array::{last_index, split};
use utils::brain;
use utils::expr::to_call_expr;
use utils::fs::{file_exists, is_constantify, to_constants_file_path};
use utils::json::{get_constants_json, get_json_key};

pub struct TransformFold;

impl Fold for TransformFold {
    fn fold_call_expr(&mut self, call_expr: CallExpr) -> CallExpr {
        let dumped = call_expr.clone();

        event!(Level::DEBUG, "Hello World");

        if let Callee::Expr(callee) = call_expr.callee {
            let ident = callee.ident();

            if !ident.is_some() {
                return dumped;
            }

            let item = ident.unwrap();

            if !is_constantify(&item.sym) {
                return dumped;
            }

            let first_arg = call_expr.args.first().expect("missing constant path!");

            if let Expr::Ident(arg) = &*first_arg.expr {
                let constant_path = split(&arg.sym);

                if constant_path.is_empty() {
                    println!("invalid constant path");
                    return dumped;
                }

                let file = constant_path.first().unwrap();

                let mut value = &brain::insert_and_return(&file, &{
                    |file: &str| {
                        let file_path = to_constants_file_path(file);

                        if !file_exists(&file_path) {
                            return Option::None;
                        }

                        return Option::Some(get_constants_json(&file_path));
                    }
                });

                for (index, key) in constant_path.iter().enumerate() {
                    // index 0 is the file
                    if index == 0 {
                        continue;
                    }

                    value = get_json_key(value, &key);

                    if value.is_null() && index < last_index(&constant_path) {
                        println!("constant is [null]");
                        break;
                    }
                }

                return to_call_expr(&value, call_expr.span);
            }
        }

        return dumped;
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut TransformFold)
}
