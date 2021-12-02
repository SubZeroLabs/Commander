use commander::executor::builder::*;
use commander::executor::command::{Command, NaturalCommand};
use commander::executor::context::CommandContext;
use commander::executor::{Executor, CommandChildContainer};
use commander::protocol::BrigadierFlags;

fn main() -> anyhow::Result<()> {
    let (executor, nodes) = Builder::executor(Executor::<String>::new())
        .wrap(
            NodeStub::new(BrigadierFlags::new(true, false, true, false, false), "command_a".into()),
            Builder::command(Command::Natural(NaturalCommand::executable(Box::new(|ctx| {
                println!("Command A: {:?}", ctx);
                Ok(())
            }))))
                .child(
                    NodeStub::new(BrigadierFlags::new(true, false, true, false, false), "sub_1".into()),
                    Command::Natural(NaturalCommand::executable(Box::new(|ctx| {
                        println!("Sub 1: {:?}", ctx);
                        Ok(())
                    }))),
                )?,
        )?
        .child(
            NodeStub::new(BrigadierFlags::new(true, false, true, false, false), "command_b".into()),
            Command::Natural(NaturalCommand::executable(Box::new(|ctx| {
                println!("Command B: {:?}", ctx);
                Ok(())
            }))),
        )?
        .into_root_split();

    println!("Node Graph: {:#?}", nodes);

    executor.execute_context(CommandContext::create("Sender A", vec!["command_a", "arg_1"]))?.unwrap()?;
    executor.execute_context(CommandContext::create("Sender A", vec!["command_a", "sub_1"]))?.unwrap()?;
    executor.execute_context(CommandContext::create("Sender A", vec!["command_b", "arg_1", "arg_2"]))?.unwrap()?;

    Ok(())
}