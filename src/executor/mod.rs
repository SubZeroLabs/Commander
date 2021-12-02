use crate::executor::command::{Command, Next};
use crate::executor::context::CommandContext;

pub mod context;
pub mod macros;
pub mod command;
pub mod builder;

pub trait CommandChildContainer<T> {
    fn child<S: Into<String>>(&mut self, identifier: S, command: command::Command<T>) -> anyhow::Result<()>;

    fn execute_context(&self, context: context::CommandContext<T>) -> anyhow::Result<Option<anyhow::Result<()>>>;
}

#[derive(Default)]
pub struct Executor<T> {
    next: command::Next<T>,
}

impl<T> Executor<T> {
    pub fn new() -> Self {
        Self { next: Next::default() }
    }
}

impl<T> CommandChildContainer<T> for Executor<T> {
    fn child<S: Into<String>>(&mut self, identifier: S, command: Command<T>) -> anyhow::Result<()> {
        match command {
            Command::Natural(_) => {
                match &mut self.next {
                    Next::LiteralMap(map) => {
                        map.insert(identifier.into(), command);
                    }
                    _ => unreachable!(),
                }
                Ok(())
            }
            Command::ArgParser(_) => anyhow::bail!("Cannot bind an arg parser at the root level."),
        }
    }

    fn execute_context(&self, mut context: CommandContext<T>) -> anyhow::Result<Option<anyhow::Result<()>>> {
        match &self.next {
            Next::LiteralMap(map) => {
                Ok(if let Some(arg) = context.args().first() {
                    if let Some(command) = map.get(arg) {
                        context.trim_top();
                        command.execute_context(context)?
                    } else {
                        None
                    }
                } else {
                    None
                })
            }
            _ => unreachable!(),
        }
    }
}
