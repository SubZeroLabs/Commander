use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

pub enum SubArg<T> {
    Empty(CommandContext<T>),
    Resolved((String, CommandContext<T>)),
}

pub type CommandFunction<T> = Box<dyn Fn(CommandContext<T>) -> anyhow::Result<()> + Send>;

// mapper
#[derive(Default)]
pub struct CommandMapper<T> {
    commands: HashMap<String, Command<T>>,
}

impl<T> CommandMapper<T> {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn add_command(&mut self, command_name: String, command: Command<T>) {
        self.commands.insert(command_name, command);
    }

    pub fn resolve_command<S: Into<String>>(&self, next_arg: S) -> Option<&Command<T>> {
        self.commands.get(&next_arg.into().to_ascii_lowercase())
    }
}

// command
#[derive(Default)]
pub struct Command<T> {
    command_function: Option<CommandFunction<T>>,
    command_mapper: CommandMapper<T>,
}

impl<T> Command<T> {
    pub fn new(func: CommandFunction<T>) -> Self {
        Self {
            command_function: Some(func),
            command_mapper: CommandMapper::new(),
        }
    }

    pub fn add_command(&mut self, command_name: String, command: Command<T>) {
        self.command_mapper.add_command(command_name, command);
    }

    pub fn execute(&self, context: CommandContext<T>) -> anyhow::Result<()> {
        let sub_arg = context.sub_arg();
        match sub_arg {
            SubArg::Empty(context) => {
                if let Some(func) = self.command_function.as_ref() {
                    func(context)
                } else {
                    anyhow::bail!("Command at this level was invalid.");
                }
            }
            SubArg::Resolved((sub, context)) => match self.command_mapper.resolve_command(&sub) {
                Some(command) => command.execute(context),
                None => {
                    if let Some(func) = self.command_function.as_ref() {
                        func(CommandContext::<T>::concat(sub, context))
                    } else {
                        anyhow::bail!("Command at this level was invalid.");
                    }
                }
            },
        }
    }
}

pub struct CommandContext<T> {
    sender: T,
    args: Vec<String>,
}

impl<T> CommandContext<T> {
    pub fn new(sender: T, args: Vec<String>) -> Self {
        Self { sender, args }
    }

    pub fn sender(&self) -> &T {
        self.sender.borrow()
    }

    pub fn args(&self) -> &Vec<String> {
        self.args.borrow()
    }

    pub fn sender_mut(&mut self) -> &mut T {
        self.sender.borrow_mut()
    }

    pub fn args_mut(&mut self) -> &mut Vec<String> {
        self.args.borrow_mut()
    }

    pub fn split(self) -> (T, Vec<String>) {
        (self.sender, self.args)
    }

    pub fn concat(arg: String, other: CommandContext<T>) -> Self {
        Self {
            sender: other.sender,
            args: [vec![arg], other.args].concat(),
        }
    }

    pub fn sub_arg(self) -> SubArg<T> {
        let mut iter = self.args.into_iter();
        let arg = iter.next();
        if let Some(arg) = arg {
            SubArg::Resolved((
                arg,
                Self {
                    sender: self.sender,
                    args: iter.collect(),
                },
            ))
        } else {
            SubArg::Empty(Self {
                sender: self.sender,
                args: Vec::with_capacity(0),
            })
        }
    }
}

// executor
#[derive(Default)]
pub struct Executor<T> {
    command_mapper: CommandMapper<T>,
}

impl<T> Executor<T> {
    pub fn new() -> Self {
        Self {
            command_mapper: CommandMapper::new(),
        }
    }

    pub fn add_command(&mut self, command_name: String, command: Command<T>) {
        self.command_mapper.add_command(command_name, command);
    }

    pub fn execute<S: Into<T>, VS: Into<String>>(
        &self,
        sender: S,
        split_args: Vec<VS>,
    ) -> Option<anyhow::Result<()>> {
        let context = CommandContext::new(
            sender.into(),
            split_args
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<String>>(),
        );
        match context.sub_arg() {
            SubArg::Empty(_) => Some(Err(anyhow::Error::msg("Cannot execute empty arguments."))),
            SubArg::Resolved((sub, context)) => self
                .command_mapper
                .resolve_command(&sub)
                .map(move |c| c.execute(context)),
        }
    }
}
