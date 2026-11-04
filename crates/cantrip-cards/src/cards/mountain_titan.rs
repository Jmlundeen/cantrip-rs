use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, triggers};
use cantrip_oracle as oracle;

/// "{1}{R}{R}: Until end of turn, whenever you cast a black spell, put a +1/+1 counter on this creature." (CR 608.2).
fn whenever_cast_black_spell_put(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}{R}{R}"),
            whenever_cast_black_spell_put,
            "{1}{R}{R}: Until end of turn, whenever you cast a black spell, put a +1/+1 counter on this creature.",
        ),
    ],
    delayed: &[
        triggered(
            when_asked(EventMask::SPELL_CAST, triggers::you_cast_a_spell_of_color::<{ Color::Black as u8 }>),
            effects::plus_one_counter_on_this,
            "Until end of turn, whenever you cast a black spell, put a +1/+1 counter on this creature.",
        ),
    ],
    ..oracle::MOUNTAIN_TITAN
};
