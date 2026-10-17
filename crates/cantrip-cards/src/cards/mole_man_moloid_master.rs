use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;


/// The 1/1 green Minion creature token named Moloid with "Whenever this token attacks, you may mill a card."
pub static MOLOID: CardDef = CardDef {
    name: "Moloid",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Minion]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[triggered(triggers::THIS_ATTACKS, effects::may_mill_a_card, "Whenever this token attacks, you may mill a card.")],
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
    abilities: &[triggered(triggers::LANDFALL, make_moloid, "Landfall \u{2014} Whenever a land you control enters, create a 1/1 green Minion creature token named Moloid with \"Whenever this token attacks, you may mill a card.\"")],
    ..oracle::MOLE_MAN_MOLOID_MASTER
};

pub static EXTRA: &[&CardDef] = &[&MOLOID];
