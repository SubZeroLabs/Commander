#[macro_export]
macro_rules! __command {
    () => {
        $crate::executor::command::Command::Natural(
            $crate::executor::command::NaturalCommand::default()
        )
    };
    (
        $sender_type:ty,
        |($($context:tt)+)| {
            $(
                $tokens:tt
            )+
        }
    ) => {
        $crate::executor::command::Command::Natural(
            $crate::executor::command::NaturalCommand::executable(Box::new(|$($context)+| {
                $(
                    $tokens
                )+
            }))
        )
    };
    (
        {$(
            $identifier:tt
        )+};
        |($($arg_context:tt)+)| {
            $(
                $arg_tokens:tt
            )+
        }
    ) => {
        $crate::executor::command::Command::ArgParser(
            $crate::executor::command::ArgParserCommand::non_executable(
                $($identifier)+,
                Box::new(|$($arg_context)+| {
                    $(
                        $arg_tokens
                    )+
                })
            )
        )
    };
    (
        {$(
            $identifier:tt
        )+};
        $sender_type:ty,
        |($($context:tt)+)| {
            $(
                $tokens:tt
            )+
        }
        |($($arg_context:tt)+)| {
            $(
                $arg_tokens:tt
            )+
        }
    ) => {
        $crate::executor::command::Command::ArgParser(
            $crate::executor::command::ArgParserCommand::executable(
                Box::new(|$($context)+| {
                    $(
                        $tokens
                    )+
                }),
                $($identifier)+,
                Box::new(|$($arg_context)+| {
                    $(
                        $arg_tokens
                    )+
                })
            )
        )
    };
}

#[macro_export]
macro_rules! executor {
    (($node_ident:ident, $executor_ident:ident, $sender_type:ty) =>
    $($name:ident [
        bind($target:ident)
        $(
            arg_parser(
                parser = $parser:expr;
                $(
                    suggestions_type = $suggestions_type:expr;
                )?
                |($($arg_context:tt)+)| {
                    $(
                        $arg_tokens:tt
                    )+
                }
            )
        )?
        $(
            |($($context:tt)+)| {
                $(
                    $tokens:tt
                )+
            }
        )?
    ])*) => {
        {
            use minecraft_data_types::nums::VarInt as __v_int;
            use $crate::protocol::Node as __node;
            use $crate::protocol::BrigadierFlags as __flags;
            use $crate::protocol::Parser as __parser;
            use $crate::protocol::SuggestionsType as __suggestions;

            let executor = #[allow(unused_mut)] {
                let mut $executor_ident = $crate::executor::Executor::<$sender_type>::new();

                $(
                    let mut $name = $crate::__command! {
                        $( // identifier for arg parse
                            {{
                                let _ = $parser;
                                stringify!($name).into()
                            }};
                        )?
                        $(
                            $sender_type,
                            |($($context)+)| {
                                $(
                                    $tokens
                                )+
                            }
                        )?
                        $(
                            |($($arg_context)+)| {
                                $(
                                    $arg_tokens
                                )+
                            }
                        )?
                    };
                )*

                $(
                    $crate::executor::CommandChildContainer::child::<&str>(&mut $target, stringify!($name), $name)?;
                )*

                $executor_ident
            };

            let node = #[allow(unused_mut)] {
                let mut index = 0;
                let mut $executor_ident = (__v_int::from(index), __v_int::from(0), Vec::<__v_int>::new());
                $(index += 1; let mut $name = (__v_int::from(index), __v_int::from(0), Vec::<__v_int>::new());)*
                $($target = ($target.0, $target.1 + 1, [$target.2, vec![$name.0]].concat());)*

                let mut node = vec![__node::new(
                    __flags::from(0u8),
                    ($executor_ident.1, $executor_ident.2),
                    None,
                    None,
                    None,
                    None
                )];
                $(
                    node.push({
                        let _parser: Option<__parser> = None;
                        let _suggestions_type: Option<__suggestions> = None;

                        let mut bit_map = 0x0;
                        bit_map |= 0x01; // they're all literal for now
                        $(
                            let __unused = stringify!($($context)+);
                            bit_map |= 0x04;
                        )?

                        $(
                            bit_map &= 0b11111110;
                            bit_map |= 0x02;
                            let _parser = Some($parser);
                            $(
                                bit_map |= 0x10;
                                let _suggestions_type = Some($suggestions_type);
                            )?
                        )?
                        // todo bit map is missing the redirect_node availability as well

                        __node::new(
                            bit_map.into(),
                            ($name.1, $name.2),
                            None, // todo(redirect_node), this can be done
                            Some(stringify!($name).into()),
                            _parser,
                            _suggestions_type,
                        )
                    });
                )*
                node
            };

            anyhow::Result::<($crate::executor::Executor<$sender_type>, Vec<__node>)>::Ok((executor, node))
        }
    }
}
