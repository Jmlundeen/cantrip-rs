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
    let created = g.create_tokens_attacking(&tokens, item.subject)?;
    let o = *g.obj(item.object);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    g.delayed_trigger_about(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        item.object,
        item.controller,
        AbilityRef::delayed(card, 0),
        &created,
    );
    Ok(())
}

/// Mobilize N itself; `text` is the card's own reminder text.
pub const fn trigger<const N: u8>(text: &'static str) -> AbilityDef {
    AbilityDef::Triggered(TriggeredDef {
        text,
        listens: EventMask::ATTACKS,
        matches: attacks,
        effect: create_warriors::<N>,
        ..TriggeredDef::BLANK
    })
}

/// Sacrifices the surviving tokens in one action; a token another player controls now cannot be sacrificed by this
/// ability's controller.
fn sacrifice_warriors(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let survivors: Vec<ObjectId> = g
        .event_objects(item)
        .iter()
        .copied()
        .filter(|&id| is_live(g, id) && g.is_on_battlefield(id) && g.controller(id) == item.controller)
        .collect();
    g.sacrifice(&survivors);
    Ok(())
}

pub const SACRIFICE: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "Sacrifice the tokens created by mobilize.",
    effect: sacrifice_warriors,
    ..TriggeredDef::BLANK
});
