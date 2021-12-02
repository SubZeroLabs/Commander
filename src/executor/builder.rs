use crate::executor::Executor;
use crate::protocol::{Node, BrigadierFlags};
use minecraft_data_types::nums::VarInt;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct NodeStub {
    flags: crate::protocol::BrigadierFlags,
    name: Option<crate::protocol::NodeName>,
    parser: Option<crate::protocol::Parser>,
    suggestions_type: Option<crate::protocol::SuggestionsType>,
}

impl NodeStub {
    pub fn new(flags: crate::protocol::BrigadierFlags, name: crate::protocol::NodeName) -> Self {
        Self {
            flags,
            name: Some(name),
            parser: None,
            suggestions_type: None,
        }
    }

    pub fn parser(flags: crate::protocol::BrigadierFlags, name: crate::protocol::NodeName, parser: crate::protocol::Parser) -> Self {
        Self {
            flags,
            name: Some(name),
            parser: Some(parser),
            suggestions_type: None,
        }
    }

    pub fn suggestion_parser(flags: crate::protocol::BrigadierFlags, name: crate::protocol::NodeName, parser: crate::protocol::Parser, suggestions_type: crate::protocol::SuggestionsType) -> Self {
        Self {
            flags,
            name: Some(name),
            parser: Some(parser),
            suggestions_type: Some(suggestions_type),
        }
    }
}

#[derive(Default, Debug)]
pub struct Builder<T, CC: super::CommandChildContainer<T>> {
    root: CC,
    commands: Vec<BuilderCommand>,
    node_index: i32,
    __phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct BuilderCommand {
    node_stub: NodeStub,
    index: i32,
    parent: i32,
}

impl<T, CC: super::CommandChildContainer<T>> Builder<T, CC> {
    pub fn wrap(mut self, sub_node_stub: NodeStub, sub_builder: Builder<T, super::command::Command<T>>) -> anyhow::Result<Self> {
        let commands_len = sub_builder.commands.len();
        self.root.child(sub_node_stub.name.as_ref().unwrap(), sub_builder.root)?;
        self.commands.push(BuilderCommand {
            node_stub: sub_node_stub,
            parent: 0,
            index: self.node_index,
        });
        for mut x in sub_builder.commands {
            x.parent += self.node_index;
            x.index += self.node_index;
            self.commands.push(x);
        }
        self.node_index += i32::try_from(commands_len)? + 1;
        Ok(self)
    }

    pub fn child(mut self, node_stub: NodeStub, command: super::command::Command<T>) -> anyhow::Result<Self> {
        self.root.child(node_stub.name.as_ref().unwrap(), command)?;
        self.commands.push(BuilderCommand {
            node_stub,
            index: self.node_index,
            parent: 0,
        });
        self.node_index += 1;
        Ok(self)
    }
}

impl<T> Builder<T, super::command::Command<T>> {
    pub fn command(command: super::command::Command<T>) -> Self {
        Self {
            root: command,
            commands: Vec::new(),
            node_index: 1,
            __phantom: PhantomData,
        }
    }
}

impl<T> Builder<T, super::Executor<T>> {
    pub fn executor(executor: super::Executor<T>) -> Self {
        Self {
            root: executor,
            commands: Vec::new(),
            node_index: 1,
            __phantom: PhantomData,
        }
    }

    pub fn into_root_split(self) -> (Executor<T>, Vec<Node>) {
        let root = self.root;

        struct InnerNode {
            node: NodeStub,
            children: Vec<VarInt>,
        }
        impl InnerNode {
            fn new(sub: NodeStub) -> Self {
                Self {
                    node: sub,
                    children: Vec::new(),
                }
            }

            fn into_node(self) -> Node {
                Node::new(
                    self.node.flags,
                    (VarInt::try_from(self.children.len()).expect("Children length should always fit into a var int size."), self.children),
                    None,
                    self.node.name,
                    self.node.parser,
                    self.node.suggestions_type,
                )
            }
        }

        let mut map = HashMap::new();
        map.insert(0, InnerNode::new(NodeStub {
            flags: BrigadierFlags::new(false, false, false, false, false),
            name: None,
            parser: None,
            suggestions_type: None,
        }));
        for command in self.commands {
            map.insert(command.index, InnerNode::new(command.node_stub));
            map.get_mut(&command.parent).unwrap().children.push(VarInt::from(command.index));
        }

        let mut nodes = map.into_iter().map(|(index, node)| {
            (index, node.into_node())
        }).collect::<Vec<(i32, Node)>>();
        nodes.sort_by(|item, other| item.0.cmp(&other.0));
        let nodes = nodes.into_iter().map(|item| item.1).collect();

        (root, nodes)
    }
}
