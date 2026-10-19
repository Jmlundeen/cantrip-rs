use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS_OR_ATTACKS, YOU_DRAW_YOUR_SECOND_CARD};
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Exile up to one target card from a graveyard. If a creature card is exiled this way, create a Clue token." The card's type
/// is read before it leaves (CR 400.7, 608.2h); the Clue follows only a creature card that really arrived in exile (CR 608.2c:
/// "exiled this way"), and it is created, which is not investigating (CR 701.16a, the ruling). With no target chosen, or the
/// target illegal on resolution, nothing is exiled.
fn exile_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let creature = g.chars(card).is(CardType::Creature);
    let exiled = g.exile(card);
    if creature && g.obj(exiled).zone.is(ZoneKind::Exile) {
        tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS_OR_ATTACKS,
            exile_a_card,
            "Whenever this creature enters or attacks, exile up to one target card from a graveyard. If a creature card is exiled this way, create a Clue token.",
        )
        .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
        triggered(
            YOU_DRAW_YOUR_SECOND_CARD,
            effects::each_opponent_loses_1_life_you_gain_1,
            "Whenever you draw your second card each turn, each opponent loses 1 life and you gain 1 life.",
        ),
    ],
    ..oracle::RAVEN_EAGLE
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
