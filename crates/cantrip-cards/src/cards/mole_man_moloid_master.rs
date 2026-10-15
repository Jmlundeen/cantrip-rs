use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "You may mill a card": asked first, then the mill (CR 701.17a).
fn may_mill(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        g.mill(item.controller, 1);
    }
    Ok(())
}

/// The 1/1 green Minion creature token named Moloid with "Whenever this token attacks, you may mill a card."
pub static MOLOID: CardDef = CardDef {
    name: "Moloid",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Minion]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever this token attacks, you may mill a card.",
        listens: EventMask::ATTACKS,
        matches: triggers::this_attacks,
        effect: may_mill,
        ..TriggeredDef::BLANK
    })],
    ..CardDef::BLANK
};

/// Landfall: create a Moloid.
fn make_moloid(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &MOLOID, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "You may play lands from your graveyard" (CR 305.1): each still uses a land play (CR 305.2).
    play_rules: &[PlayRule::LandsFromGraveyard],
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Landfall \u{2014} Whenever a land you control enters, create a 1/1 green Minion creature token named Moloid with \"Whenever this token attacks, you may mill a card.\"",
        listens: EventMask::ZONE_CHANGE,
        matches: triggers::landfall,
        effect: make_moloid,
        ..TriggeredDef::BLANK
    })],
    ..oracle::MOLE_MAN_MOLOID_MASTER
};

pub static EXTRA: &[&CardDef] = &[&MOLOID];
