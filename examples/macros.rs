#![feature(stmt_expr_attributes)]

use commander::{executor::{CommandChildContainer, context::{CommandContext, Value}}};

fn main() -> anyhow::Result<()> {
    let (local_executor, _) = commander::executor! { (node, local_executor, String) =>
        sub_1 [
            bind(parse_a)
            |(context)| {
                println!("SUB 1: Context: {:?}", context);
                Ok(())
            }
        ]

        sub_2 [
            bind(parse_a)
            |(context)| {
                println!("SUB 2: Context: {:?}", context);
                Ok(())
            }
        ]

        parse_a [
            bind(command_a)

            arg_parser(
                parser = commander::protocol::Parser::IntRange;
                |(mut context)| {
                    if let Some(arg) = context.args().first() {
                        let value = Value::Integer(arg.parse()?);
                        context.trim_top();
                        Ok((value, context))
                    } else {
                        anyhow::bail!("Cannot understand integer argument.");
                    }
                }
            )

            |(context)| {
                println!("PRS A: Context: {:?}", context);
                Ok(())
            }
        ]

        command_a [
            bind(local_executor)
        ]
    }?;

    local_executor.execute_context(CommandContext::create(
        "Some Sender",
        vec!["command_a", "123"],
    ))?;
    local_executor.execute_context(CommandContext::create(
        "Some Sender",
        vec!["command_a", "124", "sub_1", "arg_1"],
    ))?;
    local_executor.execute_context(CommandContext::create(
        "Some Sender",
        vec!["command_a", "125", "sub_1"],
    ))?;
    local_executor.execute_context(CommandContext::create(
        "Some Sender",
        vec!["command_a", "126", "sub_2"],
    ))?;

    Ok(())
}