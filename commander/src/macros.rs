macro_rules! __command {
    () => {
        {
            let command = $crate::executor::Command::default();
            command
        }
    };
    ($sender_type:ty, |$context:ident| {$($tokens:tt)+}) => {
        {
            let command = $crate::executor::Command::new(Box::new(|$context| {
                $($tokens)*
            }));
            command
        }
    }
}

#[macro_export]
macro_rules! executor {
    (($node_ident:ident, $executor_ident:ident) into sender $sender_type:ty =>
    $($name:ident [
        bind($target:ident)
        $(
            arg_parser(
                parser = $parser:expr,
                $(
                    suggestions_type = $suggestions_type:expr,
                )?
            )
        )?
        $(|$context:ident| {$($tokens:tt)+})?
    ])*) => {
        #[allow(unused_mut)]
        let ($executor_ident, $node_ident) = {
            use minecraft_data_types::nums::VarInt as __v_int;
            use $crate::protocol::Node as __node;
            use $crate::protocol::BrigadierFlags as __flags;
            use $crate::protocol::Parser as __parser;
            use $crate::protocol::SuggestionsType as __suggestions;

            let mut $executor_ident = $crate::executor::Executor::<$sender_type>::new();

            $(
                let mut $name = __command!($($sender_type, |$context| { $($tokens)+ })?);
            )*

            $(
                $target.add_command(stringify!($name).into(), $name);
                // todo if this is an arg type, convert the command map to _always_ map
                //  and push declared arguments
            )*

            let executor = $executor_ident;

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
                        let __unused = stringify!($context);
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
                    // todo bit map is very limited for now, we'll add more later

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

            (executor, node)
        };
    }
}
