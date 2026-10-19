use cantrip_api::*;
use cantrip_mechanics::{effects, empower, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker card with mana value 6 or less in your graveyard" (CR 115.1, 202.3): a card of
/// either type in the graveyard, with a mana value the spell's own effect does not change, judged as it is chosen.
fn creature_or_planeswalker_mv6(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature, CardType::Planeswalker])
        && matches!(t, Target::Object(id) if q.mana_value(id) <= 6)
}

/// "Return target creature or planeswalker card ... to the battlefield. Empower Jace 2." The card returns if its target
/// is still legal (CR 608.2b), then the empower is its own instruction.
fn rewrite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_battlefield(g, item)?;
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_or_planeswalker_mv6)], rewrite),
    ..oracle::REWRITE_REGRETS
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
