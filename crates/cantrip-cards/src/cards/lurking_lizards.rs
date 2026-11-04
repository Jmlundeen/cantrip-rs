use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::YOU_CAST_A_SPELL_WITH_MANA_VALUE_4_OR_GREATER;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_CAST_A_SPELL_WITH_MANA_VALUE_4_OR_GREATER,
        effects::plus_one_counter_on_this,
        "Whenever you cast a spell with mana value 4 or greater, put a +1/+1 counter on this creature.",
    )],
    ..oracle::LURKING_LIZARDS
};
