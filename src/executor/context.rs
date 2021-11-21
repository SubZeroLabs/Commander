#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(u32),
    Float(f32),
    Generic(String),
}

#[macro_export]
macro_rules! unwrap_value {
    ($value:expr => $as_type:ident) => {
        if let Value::$as_type(inner) = $value { inner } else { unreachable!(); }
    }
}

pub type ArgValue = (String, Value);

#[derive(Debug)]
pub struct ArgValues {
    values: Vec<ArgValue>,
}

impl ArgValues {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn into_inner(self) -> Vec<ArgValue> {
        self.values
    }

    pub fn borrow_inner(&self) -> &Vec<ArgValue> {
        &self.values
    }
}

#[derive(Debug)]
pub struct CommandContext<T> {
    sender: T,
    args: Vec<String>,
    value_args: ArgValues,
}

impl<T> CommandContext<T> {
    pub fn create<S: Into<T>, VS: Into<String>>(sender: S, split_args: Vec<VS>) -> Self {
        Self::new(
            sender.into(),
            split_args
                .into_iter()
                .map(|item| item.into())
                .collect::<Vec<String>>(),
        )
    }

    pub fn new(sender: T, args: Vec<String>) -> Self {
        Self { sender, args, value_args: ArgValues::new() }
    }

    pub fn sender(&self) -> &T {
        &self.sender
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn sender_mut(&mut self) -> &mut T {
        &mut self.sender
    }

    pub fn args_mut(&mut self) -> &mut Vec<String> {
        &mut self.args
    }

    pub fn split(self) -> (T, Vec<String>) {
        (self.sender, self.args)
    }

    pub fn value_arg(&mut self, value: ArgValue) {
        self.value_args.values.push(value);
    }

    pub fn trim_top(&mut self) {
        self.args.remove(0);
    }

    pub fn overwrite_args(&mut self, new_args: Vec<String>) {
        self.args = new_args;
    }
}