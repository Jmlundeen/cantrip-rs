use cantrip_api::*;
use cantrip_mechanics::triggers::this_enters;
use cantrip_mechanics::{discard_chosen, sacrifice_chosen, targets, triggers};
use cantrip_oracle as oracle;

fn creature_or_planeswalker(chars: &Chars) -> bool {
    chars.is(CardType::Creature) || chars.is(CardType::Planeswalker)
}

/// Target opponent sacrifices a creature or planeswalker of their choice, discards a card, and loses 3 life. You draw a
/// card and gain 3 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    sacrifice_chosen(g, opponent, item.object, 1, creature_or_planeswalker)?;
    discard_chosen(g, opponent, item.object, 1)?;
    g.lose_life(opponent, 3);
    g.draw(item.controller, 1);
    g.gain_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Archon of Cruelty enters, target opponent sacrifices a creature or planeswalker of their choice, discards a card, and loses 3 life. You draw a card and gain 3 life.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::OPPONENT],
            effect: drain,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Archon of Cruelty attacks, target opponent sacrifices a creature or planeswalker of their choice, discards a card, and loses 3 life. You draw a card and gain 3 life.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            targets: &[targets::OPPONENT],
            effect: drain,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::ARCHON_OF_CRUELTY
};
