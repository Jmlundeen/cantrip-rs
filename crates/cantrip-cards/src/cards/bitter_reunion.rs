use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::this_enters};
use cantrip_oracle as oracle;

/// "You may discard a card. If you do, draw two cards." Choosing no card declines; the draw follows a card that was
/// actually discarded. Nothing is asked of an empty hand.
fn discard_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    if let Some(&card) = choose::cards(g, item.controller, item.object, Purpose::Discard, hand, 1, true)?.first() {
        g.discard(card);
        g.draw(item.controller, 2);
    }
    Ok(())
}

/// Creatures you control gain haste until end of turn.
fn haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        g.add_effect(item.object, id, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Bitter Reunion enters, you may discard a card. If you do, draw two cards.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: discard_to_draw,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, Sacrifice Bitter Reunion: Creatures you control gain haste until end of turn.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
            effect: haste,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::BITTER_REUNION
};
