use minecraft_data_types::common::Identifier;
use minecraft_data_types::encoder::{AsyncEncodable, Decodable, Encodable};
use minecraft_data_types::nums::VarInt;
use minecraft_data_types::strings::McString;
use std::io::{Read, Write};
use tokio::io::AsyncWrite;

macro_rules! bit_map {
    ($($map_name:ident {
        $(
            $option_name:ident from $bit_field:literal;
        )+
    })+) => {$(
        #[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
        pub struct $map_name {
            $(
                $option_name: bool,
            )*
        }

        impl $map_name {
            pub fn new($($option_name: bool,)*) -> Self {
                Self { $($option_name,)* }
            }
        }

        impl From<u8> for $map_name {
            fn from(byte: u8) -> Self {
                Self {
                    $(
                        $option_name: byte & $bit_field != 0x0,
                    )+
                }
            }
        }

        impl Encodable for $map_name {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> anyhow::Result<()> {
                let mut byte = 0x0;
                $(
                    if self.$option_name {
                        byte |= $bit_field;
                    }
                )+
                byte.encode(writer)
            }

            fn size(&self) -> anyhow::Result<VarInt> {
                Ok(VarInt::from(1))
            }
        }

        impl Decodable for $map_name {
            fn decode<R: std::io::Read>(reader: &mut R) -> anyhow::Result<Self> {
                let byte = u8::decode(reader)?;
                Ok(<$map_name>::from(byte))
            }
        }

        #[async_trait::async_trait]
        impl AsyncEncodable for $map_name {
            async fn async_encode<W: AsyncWrite + Send + Unpin>(&self, writer: &mut W) -> anyhow::Result<()> {
                let mut byte = 0x0;
                $(
                    if self.$option_name {
                        byte |= $bit_field;
                    }
                )+
                byte.async_encode(writer).await
            }
        }
    )*}
}

macro_rules! parser {
    ($(
        $identifier:literal as $enum_identifier:ident $(
            => BitProperties: $bit_field:ty $(
                | ($bit_property_ident:ident, $bit_property_type:ty, $bit_option_ident:ident)
            )+
        )? $(
            => Properties $(
                | ($property_ident:ident, $property_type:ty)
            )+
        )?;
    )+) => {
        #[derive(Debug)]
        pub enum Parser {
            $(
                $enum_identifier$(
                    { bits: $bit_field, $(
                        $bit_property_ident: Option<$bit_property_type>,
                    )+}
                )? $(
                    {$(
                        $property_ident: $property_type
                    )+}
                )?,
            )+
        }

        impl Encodable for Parser {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> anyhow::Result<()> {
                match self {
                    $(
                        Parser::$enum_identifier
                        $({ bits, $(
                            $bit_property_ident,
                        )+})?
                        $({$(
                            $property_ident,
                        )+})? => {
                            Identifier::from($identifier).encode(writer)?;
                            $(
                                bits.encode(writer)?;
                                $(
                                    $bit_property_ident.encode(writer)?;
                                )+
                            )?
                            $($(
                                $property_ident.encode(writer)?;
                            )+)?
                            Ok(())
                        }
                    )+
                }
            }

            fn size(&self) -> anyhow::Result<VarInt> {
                match self {
                    $(
                        Parser::$enum_identifier
                        $({ bits, $(
                            $bit_property_ident,
                        )+})?
                        $({$(
                            $property_ident,
                        )+})? => {
                            Ok(Identifier::from($identifier).size()?
                                $( + bits.size()? $(
                                    + $bit_property_ident.size()?
                                )+)?
                                $($(
                                    + $property_ident.size()?
                                )+)?
                            )
                        }
                    )+
                }
            }
        }

        #[async_trait::async_trait]
        impl AsyncEncodable for Parser {
            async fn async_encode<W: AsyncWrite + Send + Unpin>(&self, writer: &mut W) -> anyhow::Result<()> {
                match self {
                    $(
                        Parser::$enum_identifier
                        $({ bits, $(
                            $bit_property_ident,
                        )+})?
                        $({$(
                            $property_ident,
                        )+})? => {
                            Identifier::from($identifier).async_encode(writer).await?;
                            $(
                                bits.async_encode(writer).await?;
                                $(
                                    $bit_property_ident.async_encode(writer).await?;
                                )+
                            )?
                            $($(
                                $property_ident.async_encode(writer).await?;
                            )+)?
                            Ok(())
                        }
                    )+
                }
            }
        }

        impl Decodable for Parser {
            fn decode<R: std::io::Read>(reader: &mut R) -> anyhow::Result<Self> {
                let ident = Identifier::decode(reader)?;
                match ident.string().as_str() {
                    $(
                        $identifier => {
                            $(
                                let bit_field = <$bit_field>::decode(reader)?;
                                $(
                                    let $bit_property_ident = if bit_field.$bit_option_ident {
                                        Some(<$bit_property_type>::decode(reader)?)
                                    } else {
                                        None
                                    };
                                )+
                            )?
                            $($(
                                let $property_ident = <$property_type>::decode(reader)?;
                            )+)?
                            Ok(Parser::$enum_identifier
                                $({ bits: bit_field, $(
                                   $bit_property_ident,
                                )+})?
                                $({$(
                                    $property_ident,
                                )+})?
                            )
                        },
                    )+
                    _ => anyhow::bail!("Unknown identifier: {:?}", &ident),
                }
            }
        }
    }
}

bit_map! {
    BrigadierFlags {
        node_literal from 0x01;
        node_argument from 0x02;
        executable from 0x04;
        has_redirect from 0x08;
        has_suggestions_type from 0x10;
    }

    MinMax {
        min from 0x01;
        max from 0x02;
    }

    EntitySelector {
        entity_or_player from 0x01;
        players_only from 0x02;
    }

    ScoreHolderSelector {
        multiple from 0x01;
    }
}

mc_packet_protocol::strict_enum! {
    StringDescription; minecraft_data_types::nums::VarInt {
        0 => SingleWord;
        1 => QuotablePhrase;
        2 => GreedyPhrase;
    }

    SuggestionsType; minecraft_data_types::common::Identifier {
        "minecraft:ask_server" => AskServer;
        "minecraft:all_recipes" => AllRecipes;
        "minecraft:available_sounds" => AvailableSounds;
        "minecraft:available_biomes" => AvailableBiomes;
        "minecraft:summonable_entities" => SummonableEntities;
    }
}

impl BrigadierFlags {
    pub fn is_root(&self) -> bool {
        !self.node_argument && !self.node_literal
    }

    pub fn is_literal(&self) -> bool {
        self.node_literal
    }

    pub fn is_argument(&self) -> bool {
        self.node_argument
    }
}

parser! {
    "brigadier:bool" as Bool;
    "brigadier:double" as Double => BitProperties: MinMax | (min, f64, min) | (max, f64, max);
    "brigadier:float" as Float => BitProperties: MinMax | (min, f32, min) | (max, f32, max);
    "brigadier:integer" as Integer => BitProperties: MinMax | (min, i32, min) | (max, i32, max);
    "brigadier:long" as Long => BitProperties: MinMax | (min, i64, min) | (max, i64, max);
    "brigadier:string" as String => Properties | (info, StringDescription);
    "minecraft:entity" as Entity => Properties | (selector, EntitySelector);
    "minecraft:game_profile" as GameProfile;
    "minecraft:block_pos" as BlockPos;
    "minecraft:column_pos" as ColumnPos;
    "minecraft:vec3" as Vec3;
    "minecraft:vec2" as Vec2;
    "minecraft:block_state" as BlockState;
    "minecraft:block_predicate" as BlockPredicate;
    "minecraft:item_stack" as ItemStack;
    "minecraft:item_predicate" as ItemPredicate;
    "minecraft:color" as Color;
    "minecraft:component" as Component;
    "minecraft:message" as Message;
    "minecraft:nbt" as Nbt;
    "minecraft:nbt_path" as NbtPath;
    "minecraft:objective" as Objective;
    "minecraft:objective_criteria" as ObjectiveCriteria;
    "minecraft:operation" as Operation;
    "minecraft:particle" as Particle;
    "minecraft:rotation" as Rotation;
    "minecraft:angle" as Angle;
    "minecraft:scoreboard_slot" as ScoreboardSlot;
    "minecraft:score_holder" as ScoreHolder => Properties | (selector, ScoreHolderSelector);
    "minecraft:swizzle" as Swizzle;
    "minecraft:team" as Team;
    "minecraft:item_slot" as ItemSlot;
    "minecraft:resource_location" as ResourceLocation;
    "minecraft:mob_effect" as MobEffect;
    "minecraft:function" as Function;
    "minecraft:entity_anchor" as EntityAnchor;
    "minecraft:range" as Range => Properties | (decimals, bool);
    "minecraft:int_range" as IntRange;
    "minecraft:float_range" as FloatRange;
    "minecraft:item_enchantment" as ItemEnchantment;
    "minecraft:entity_summon" as EntitySummon;
    "minecraft:dimension" as Dimension;
    "minecraft:uuid" as Uuid;
    "minecraft:nbt_tag" as NbtTag;
    "minecraft:nbt_compound_tag" as NbtCompoundTag;
    "minecraft:time" as Time;
}

minecraft_data_types::auto_string!(NodeName, 32767);

#[derive(Debug)]
pub struct Node {
    flags: BrigadierFlags,
    children: (VarInt, Vec<VarInt>),
    redirect_node: Option<VarInt>,
    name: Option<NodeName>,
    parser: Option<Parser>,
    suggestions_type: Option<SuggestionsType>,
}

impl Node {
    pub fn new(
        flags: BrigadierFlags,
        children: (VarInt, Vec<VarInt>),
        redirect_node: Option<VarInt>,
        name: Option<NodeName>,
        parser: Option<Parser>,
        suggestions_type: Option<SuggestionsType>,
    ) -> Self {
        Self {
            flags,
            children,
            redirect_node,
            name,
            parser,
            suggestions_type,
        }
    }
}

impl Encodable for Node {
    fn encode<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        self.flags.encode(writer)?;
        self.children.encode(writer)?;
        if self.flags.has_redirect {
            self.redirect_node
                .as_ref()
                .expect("Redirect node should be provided if the flags are set as such.")
                .encode(writer)?;
        };
        if !self.flags.is_root() {
            self.name
                .as_ref()
                .expect("Name should be provided.")
                .encode(writer)?;
        };
        if self.flags.is_argument() {
            self.parser
                .as_ref()
                .expect("Parser should be provided.")
                .encode(writer)?;
        };
        if self.flags.has_suggestions_type {
            self.suggestions_type
                .as_ref()
                .expect("Suggestions type should be provided")
                .encode(writer)?;
        };
        Ok(())
    }

    fn size(&self) -> anyhow::Result<VarInt> {
        let mut size = VarInt::from(0);
        size += self.flags.size()?;
        size += self.children.size()?;
        if self.flags.has_redirect {
            size += self
                .redirect_node
                .as_ref()
                .expect("Redirect node should be provided if the flags are set as such.")
                .size()?;
        };
        if !self.flags.is_root() {
            size += self
                .name
                .as_ref()
                .expect("Name should be provided.")
                .size()?;
        };
        if self.flags.is_argument() {
            size += self
                .parser
                .as_ref()
                .expect("Parser should be provided.")
                .size()?;
        };
        if self.flags.has_suggestions_type {
            size += self
                .suggestions_type
                .as_ref()
                .expect("Suggestions type should be provided")
                .size()?;
        };
        Ok(size)
    }
}

#[async_trait::async_trait]
impl AsyncEncodable for Node {
    async fn async_encode<W: AsyncWrite + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        self.flags.async_encode(writer).await?;
        self.children.async_encode(writer).await?;
        if self.flags.has_redirect {
            self.redirect_node
                .as_ref()
                .expect("Redirect node should be provided if the flags are set as such.")
                .async_encode(writer)
                .await?;
        };
        if !self.flags.is_root() {
            self.name
                .as_ref()
                .expect("Name should be provided.")
                .async_encode(writer)
                .await?;
        };
        if self.flags.is_argument() {
            self.parser
                .as_ref()
                .expect("Parser should be provided.")
                .async_encode(writer)
                .await?;
        };
        if self.flags.has_suggestions_type {
            self.suggestions_type
                .as_ref()
                .expect("Suggestions type should be provided")
                .async_encode(writer)
                .await?;
        };
        Ok(())
    }
}

impl Decodable for Node {
    fn decode<R: Read>(reader: &mut R) -> anyhow::Result<Self> {
        let flags = BrigadierFlags::decode(reader)?;
        Ok(Self {
            flags,
            children: <(VarInt, Vec<VarInt>)>::decode(reader)?,
            redirect_node: if flags.has_redirect {
                <Option<VarInt>>::decode(reader)?
            } else {
                None
            },
            name: if !flags.is_root() {
                <Option<NodeName>>::decode(reader)?
            } else {
                None
            },
            parser: if flags.is_argument() {
                <Option<Parser>>::decode(reader)?
            } else {
                None
            },
            suggestions_type: if flags.has_suggestions_type {
                <Option<SuggestionsType>>::decode(reader)?
            } else {
                None
            },
        })
    }
}

#[cfg(test)]
mod test {
    use crate::protocol::{BrigadierFlags, Parser};
    use minecraft_data_types::encoder::{Decodable, Encodable};
    use std::io::{Cursor, Seek};

    #[test]
    pub fn test_bit_persistence() {
        let flags = BrigadierFlags {
            node_literal: false,
            node_argument: true,
            executable: true,
            has_redirect: true,
            has_suggestions_type: false,
        };
        let mut encoder = Cursor::new(Vec::new());
        flags
            .encode(&mut encoder)
            .expect("Should encode into cursor.");
        encoder.rewind().expect("Cursor should rewind.");
        assert_eq!(
            flags,
            BrigadierFlags::decode(&mut encoder).expect("Brigadier flags should decode.")
        );
    }
}
