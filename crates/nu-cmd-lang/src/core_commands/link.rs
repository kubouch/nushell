use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use nu_engine::{eval_block, CallExt};
use nu_parser::parse_module_file_or_dir;
use nu_protocol::ast::{Block, Call, Expr, PipelineElement};
use nu_protocol::engine::{Command, EngineState, Stack, StateWorkingSet};
use nu_protocol::{
    Category, Example, PipelineData, ShellError, Signature, Spanned, SyntaxShape, Type,
};

#[derive(Clone)]
pub struct Link;

impl Command for Link {
    fn name(&self) -> &str {
        "link"
    }

    fn usage(&self) -> &str {
        "Parse a module at runtime."
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("link")
            .input_output_types(vec![(Type::Nothing, Type::Nothing)])
            .required("module", SyntaxShape::String, "module file or directory")
            .required(
                "signatures",
                SyntaxShape::List(Box::new(SyntaxShape::Signature)),
                "signatures of module's commands",
            )
            // .required(
            //     "signatures",
            //     SyntaxShape::List(Box::new(SyntaxShape::Record(vec![
            //         ("type".to_string(), SyntaxShape::String),
            //         ("name".to_string(), SyntaxShape::String),
            //         ("signature".to_string(), SyntaxShape::Signature),
            //     ]))),
            //     "signatures of module's commands",
            // )
            .category(Category::Core)
    }

    fn is_parser_keyword(&self) -> bool {
        true
    }

    fn can_link(&self) -> bool {
        true
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let file: String = call.req(engine_state, stack, 0)?;
        println!("File: {file}, Signatures: {:?}", call.positional_nth(1));
        Ok(PipelineData::empty())
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            // Example {
            //     description: "Define a command and run it",
            //     example: r#"def say-hi [] { echo 'hi' }; say-hi"#,
            //     result: Some(Value::test_string("hi")),
            // },
            // Example {
            //     description: "Define a command and run it with parameter(s)",
            //     example: r#"def say-sth [sth: string] { echo $sth }; say-sth hi"#,
            //     result: Some(Value::test_string("hi")),
            // },
        ]
    }
}

pub fn eval_block_mut(
    engine_state: &mut EngineState,
    stack: &mut Stack,
    block: &Block,
    input: PipelineData,
    redirect_stdout: bool,
    redirect_stderr: bool,
) -> Result<PipelineData, ShellError> {
    let mut decls = vec![];

    for pipeline in block.pipelines.iter() {
        if let Some(element) = pipeline.elements.first() {
            if let PipelineElement::Expression(_, expr) = element {
                if let Expr::Call(call) = &expr.expr {
                    let decl = engine_state.get_decl(call.decl_id);

                    if decl.can_link() {
                        let path: Spanned<String> = call.req(engine_state, stack, 0)?;
                        let mut working_set = StateWorkingSet::new(&engine_state);

                        let Some(module_id) = parse_module_file_or_dir(
                            &mut working_set,
                            path.item.as_bytes(),
                            path.span,
                            None,
                        ) else {
                            // TODO: Error
                            panic!("err");
                        };

                        if let Some(err) = working_set.parse_errors.first() {
                            return Err(ShellError::GenericError(
                                "Failed to parse content".to_string(),
                                format!("Error parsing module: {err}"),
                                Some(path.span),
                                Some(
                                    "Encountered errors when parsing the module '{path.item}'"
                                        .to_string(),
                                ),
                                Vec::new(),
                            ));
                        }

                        if let Some(module_name) = Path::new(&path.item).file_stem() {
                            println!("module name {}", String::from_utf8_lossy(module_name.as_bytes()));
                            if let Some(existing_module_id) = working_set.find_module(module_name.as_bytes()) {
                                println!("existing module {}", existing_module_id);
                                let module_decls = working_set.get_module(module_id).decls.clone();
                                let existing_module_decls = working_set.get_module(existing_module_id).decls.clone();

                                for (existing_decl_name, existing_decl_id) in existing_module_decls {
                                    println!("existing decl: {}, {}", String::from_utf8_lossy(&existing_decl_name), existing_decl_id);
                                    println!("ndecls: {}", working_set.num_decls());
                                    if let Some(used_name) = working_set.find_decl_name(existing_decl_id) {
                                        println!("used name: {}", String::from_utf8_lossy(&used_name));
                                        if let Some(decl_id) = module_decls.get(&existing_decl_name) {
                                            println!("decl_id: {}", decl_id);
                                            let decl = working_set.get_decl(*decl_id);
                                            if let Some(block_id) = decl.get_block_id() {
                                                println!("block_id: {}", block_id);
                                                // let mut sig = decl.signature();
                                                // sig.name = String::from_utf8_lossy(used_name).to_string();
                                                // let new_decl = sig.into_block_command(block_id);
                                                // working_set.add_decl(new_decl);

                                                decls.push((
                                                        String::from_utf8_lossy(used_name).to_string(),
                                                        *decl_id,
                                                        block_id));
                                            }
                                        }
                                    }
                                }

                            }
                        }

                        // let module_decls = working_set.get_module(module_id).decls.clone();

                        // for (_name, decl_id) in module_decls {
                        //     if
                        //     let mut decl = working_set.get_decl_mut(decl_id);

                        //     let sig = decl.signature();
                        //     lf decl.get_block_id()

                        //     *decl = signature.clone().into_block_command
                        // }

                        engine_state.merge_delta(working_set.delta)?;
                    }
                }
            }
        }
    }

    println!("{:?}", decls);
    for (name, decl_id, block_id) in decls {
        println!("Updating {}", name);
        let decl = engine_state.get_decl(decl_id);
        let mut sig = decl.signature();
        sig.name = name;
        let new_decl = sig.into_block_command(block_id);
        engine_state.update_decl(decl_id, new_decl);
    }

    eval_block(
        engine_state,
        stack,
        block,
        input,
        redirect_stdout,
        redirect_stderr,
    )
}
