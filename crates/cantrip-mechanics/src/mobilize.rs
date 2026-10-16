//! Mobilize N (CR 702.181a): "Whenever this creature attacks, create N tapped and attacking 1/1 red Warrior creature
//! tokens. Sacrifice them at the beginning of the next end step." Voice of Victory prints it with 2.
//!
//! A card with mobilize lists `trigger::<N>(text)` among its `abilities`, `SACRIFICE` as its first `delayed` ability
//! and `WARRIOR` among its `EXTRA` definitions.

use cantrip_api::*;

/// The 1/1 red Warrior creature token mobilize makes.
pub static WARRIOR: CardDef = CardDef {
    name: "Warrior",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Warrior]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// "Whenever this creature attacks": its event carries who it attacks.
fn attacks(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender } if *attacker == source => Fired::about(*defender),
        _ => None,
    }
}

/// `N` tapped and attacking Warriors, created together and attacking what the creature attacks, then sacrificed together
/// at the beginning of the next end step (CR 603.7c, 701.21a). The sacrifice is the card's delayed ability 0; it
/// remembers every token that was created, however many a replacement made of them (Elspeth, Storm Slayer doubles them).
fn create_warriors<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let warrior = g.card_id(&WARRIOR);
    let tokens: Vec<(CardId, PlayerId)> = (0..N).map(|_| (warrior, item.controller)).collect();
    // The Warriors are the keyword's (CR 702.181a), whatever the card's text says now.
    let created = g.create_tokens_worded(&tokens, true, Some(item.subject), Words::NONE)?;
    crate::delayed::at_next_end_step_about(g, item, 0, &created);
    Ok(())
}

/// Mobilize N itself; `text` is the card's own reminder text.
pub const fn trigger<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::ATTACKS, attacks), create_warriors::<N>, text)
}

pub const SACRIFICE: AbilityDef = delayed_trigger(crate::delayed::sacrifice_created, "Sacrifice the tokens created by mobilize.");
