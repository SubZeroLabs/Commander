use super::context::{CommandContext, Value};
use std::collections::HashMap;
use std::borrow::{BorrowMut, Borrow};
use crate::executor::CommandChildContainer;

pub type CommandFunction<T> = Box<dyn Fn(CommandContext<T>) -> anyhow::Result<()> + Send + Sync>;
pub type ParserFunction<T> = Box<dyn Fn(CommandContext<T>) -> anyhow::Result<(Value, CommandContext<T>)> + Send + Sync>;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub enum Next<T> {
    ArgumentParser(Command<T>),
    LiteralMap(HashMap<String, Command<T>>),
}

impl<T> Default for Next<T> {
    fn default() -> Self {
        Next::LiteralMap(HashMap::new())
    }
}

pub enum Command<T> {
    Natural(NaturalCommand<T>),
    ArgParser(ArgParserCommand<T>),
}

impl<T> Command<T> {
    fn __child(next: &mut Next<T>, identifier: String, command: Command<T>) -> anyhow::Result<Option<Box<Next<T>>>> {
        match next {
            Next::ArgumentParser(_) => anyhow::bail!("Cannot assign a child to a command with a arg parser child."),
            Next::LiteralMap(map) => {
                match command {
                    Command::Natural(_) => {
                        map.insert(identifier, command);
                        Ok(None)
                    }
                    Command::ArgParser(_) => {
                        Ok(Option::Some(Box::new(Next::ArgumentParser(command))))
                    }
                }
            }
        }
    }

    fn __next(next: &Next<T>, mut context: CommandContext<T>) -> anyhow::Result<Either<CommandContext<T>, Option<anyhow::Result<()>>>> {
        match next {
            Next::ArgumentParser(parser_command) => {
                match parser_command {
                    Command::Natural(_) => unreachable!(),
                    Command::ArgParser(parser) => {
                        let resolved_context: CommandContext<T> = parser.parse(context)?;
                        Ok(Either::Right(Command::execute_context(parser_command, resolved_context)?))
                    }
                }
            }
            Next::LiteralMap(sub_commands) => {
                if let Some(first_subbed) = context.args().first() {
                    if let Some(command) = sub_commands.get(first_subbed) {
                        context.trim_top();
                        Ok(Either::Right(Command::execute_context(command, context)?))
                    } else {
                        Ok(Either::Left(context))
                    }
                } else {
                    Ok(Either::Left(context))
                }
            }
        }
    }

    fn __execute(function: &Option<CommandFunction<T>>, context: CommandContext<T>) -> Option<anyhow::Result<()>> {
        function.as_ref().map(|f| f(context))
    }
}

impl<T> super::CommandChildContainer<T> for Command<T> {
    fn child(&mut self, identifier: String, command: Command<T>) -> anyhow::Result<()> {
        match self {
            Command::Natural(inner) => {
                if let Some(new_next) = Command::__child(inner.next.borrow_mut(), identifier, command)? {
                    inner.next = new_next;
                }
                Ok(())
            }
            Command::ArgParser(inner) => {
                if let Some(new_next) = Command::__child(inner.next.borrow_mut(), identifier, command)? {
                    inner.next = new_next;
                }
                Ok(())
            }
        }
    }

    fn execute_context(&self, context: CommandContext<T>) -> anyhow::Result<Option<anyhow::Result<()>>> {
        match self {
            Command::Natural(inner) => {
                match Command::__next(inner.next.borrow(), context)? {
                    Either::Left(passed_back) => {
                        Ok(Command::__execute(&inner.command_function, passed_back))
                    }
                    Either::Right(pushed) => {
                        Ok(pushed)
                    }
                }
            }
            Command::ArgParser(inner) => {
                match Command::__next(inner.next.borrow(), context)? {
                    Either::Left(passed_back) => {
                        Ok(Command::__execute(&inner.command_function, passed_back))
                    }
                    Either::Right(pushed) => {
                        Ok(pushed)
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct NaturalCommand<T> {
    command_function: Option<CommandFunction<T>>,
    next: Box<Next<T>>,
}

impl<T> NaturalCommand<T> {
    pub fn executable(command_function: CommandFunction<T>) -> Self {
        Self { command_function: Some(command_function), next: Box::new(Next::default()) }
    }
}

pub struct ArgParserCommand<T> {
    command_function: Option<CommandFunction<T>>,
    identifier: String,
    parser_function: ParserFunction<T>,
    next: Box<Next<T>>,
}

impl<T> ArgParserCommand<T> {
    pub fn non_executable(identifier: String, parser_function: ParserFunction<T>) -> Self {
        Self { command_function: None, identifier, parser_function, next: Box::new(Next::default()) }
    }

    pub fn executable(command_function: CommandFunction<T>, identifier: String, parser_function: ParserFunction<T>) -> Self {
        Self { command_function: Some(command_function), identifier, parser_function, next: Box::new(Next::default()) }
    }

    pub fn parse(&self, context: CommandContext<T>) -> anyhow::Result<CommandContext<T>> {
        let (value, mut new_context) = (&self.parser_function)(context)?;
        new_context.value_arg((self.identifier.clone(), value));
        Ok(new_context)
    }
}
