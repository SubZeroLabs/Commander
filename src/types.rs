use minecraft_data_types::encoder::{Encodable, Decodable};
use minecraft_data_types::nums::VarInt;
use minecraft_data_types::common::Identifier;
use anyhow::Context;
use minecraft_data_types::strings::McString;

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

        impl Encodable for $map_name {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> anyhow::Result<()> {
                let mut byte = 0x0;
                $(
                    if self.$option_name {
                        byte |= $bit_field;
                    }
                )+
                writer.write_all(&[byte]).context("Failed to write bit byte into writer.")
            }

            fn size(&self) -> anyhow::Result<VarInt> {
                Ok(VarInt::from(1))
            }
        }

        impl Decodable for $map_name {
            fn decode<R: std::io::Read>(reader: &mut R) -> anyhow::Result<Self> {
                let byte = u8::decode(reader)?;
                Ok(Self {
                    $(
                        $option_name: byte & $bit_field != 0x0,
                    )+
                })
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
        had_suggestions_type from 0x10;
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

pub struct Node {
    flags: BrigadierFlags,
    children: (VarInt, Vec<VarInt>),
    redirect_node: Option<VarInt>,
    name: Option<NodeName>,
    parser: Option<Parser>,
    suggestions_type: Option<SuggestionsType>,
}

#[cfg(test)]
mod test {
    use crate::types::{BrigadierFlags, Parser};
    use std::io::{Cursor, Seek};
    use minecraft_data_types::encoder::{Encodable, Decodable};

    #[test]
    pub fn test_bit_persistence() {
        let flags = BrigadierFlags {
            node_literal: false,
            node_argument: true,
            executable: true,
            has_redirect: true,
            had_suggestions_type: false,
        };
        let mut encoder = Cursor::new(Vec::new());
        flags.encode(&mut encoder).expect("Should encode into cursor.");
        encoder.rewind().expect("Cursor should rewind.");
        assert_eq!(flags, BrigadierFlags::decode(&mut encoder).expect("Brigadier flags should decode."));
    }
}
