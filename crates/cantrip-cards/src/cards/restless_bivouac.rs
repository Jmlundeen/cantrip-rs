use cantrip_api::*;
use cantrip_mechanics::{effects, lands, mana, targets, triggers};
use cantrip_oracle as oracle;

/// "{1}{R}{W}: This land becomes a 2/2 red and white Ox creature until end of turn. It's still a land." (CR 613.1d, 613.4b, 611.2a)
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let colors = ColorSet(ColorSet::of(Color::Red).0 | ColorSet::of(Color::White).0);
    lands::becomes_creature_until_end_of_turn(g, item, (2, 2), colors, Subtype::Ox, Keywords::NONE);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::White, Color::Red]), "{T}: Add {R} or {W}."),
        ability(
            cost("{1}{W}{R}"),
            becomes_creature,
            "{1}{R}{W}: This land becomes a 2/2 red and white Ox creature until end of turn. It's still a land.",
        ),
        triggered(
            triggers::THIS_ATTACKS,
            effects::plus_one_counter_on_target,
            "Whenever this land attacks, put a +1/+1 counter on target creature you control.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    ..oracle::RESTLESS_BIVOUAC
};
