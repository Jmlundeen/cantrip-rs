use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// “Mill two cards” (CR 701.17): mill as many as possible if fewer than two remain.
fn mill_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 2);
    Ok(())
}

/// “Target creature or planeswalker card in your graveyard” (CR 115.1).
fn creature_or_planeswalker_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature, CardType::Planeswalker])
}

/// Return the targeted card under your control (CR 110.2a), then put a +1/+1 counter on Liliana.
fn return_permanent_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_battlefield(g, item)?;
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

/// “Exhaust — {5}{B}: Return target creature or planeswalker card from your graveyard to the battlefield. Put a +1/+1
/// counter on Liliana. Activate only as a sorcery.” (CR 702.177a, 602.5d).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask::ZONE_CHANGE, triggers::another_creature_or_planeswalker_you_control_enters),
            mill_two,
            "Whenever another creature or planeswalker you control enters, mill two cards.",
        ),
        ability(
            cost("{5}{B}"),
            return_permanent_and_grow,
            "Exhaust — {5}{B}: Return target creature or planeswalker card from your graveyard to the battlefield. Put a +1/+1 counter on Liliana. Activate only as a sorcery. (Activate each exhaust ability only once.)",
        )
        .targets(&[target(creature_or_planeswalker_card_in_your_graveyard)])
        .sorcery_speed()
        .exhaust(),
    ],
    ..oracle::LILIANA_THE_REPENTANT
};
