use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets};
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_oracle as oracle;

fn pump_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 1, 0);
    g.end_sentence();
    let power = g.state().battlefield().iter()
        .filter(|&&id| g.state().obj(id).controller == item.controller && g.chars(id).is(CardType::Creature))
        .map(|&id| g.chars(id).power.unwrap_or(0))
        .sum::<i16>();
    if power >= 8 {
        instruction(g, |g| {
            effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Menace);
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(BEGINNING_OF_YOUR_COMBAT, pump_target, "At the beginning of combat on your turn, target creature you control gets +1/+0 until end of turn. Then that creature gains menace until end of turn if creatures you control have total power 8 or greater.")
        .targets(&[targets::CREATURE_YOU_CONTROL])
    ],
    ..oracle::CANTANKEROUS_CAPTAIN
};
