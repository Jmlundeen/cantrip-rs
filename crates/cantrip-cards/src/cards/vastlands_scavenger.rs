use cantrip_api::*;
use cantrip_mechanics::library::mill_then_put_onto_battlefield;
use cantrip_oracle as oracle;

/// "Mill seven cards. Then put a creature card from among them onto the battlefield." (CR 701.17, 701.17c.)
fn bind_to_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    mill_then_put_onto_battlefield(g, item.controller, item.object, 7, |chars| chars.is(CardType::Creature))
}

/// Vastlands Scavenger's prepare spell (CR 722): not a card here, only ever a copy.
pub static BIND_TO_LIFE: CardDef = CardDef {
    name: "Bind to Life",
    cost: mana("{4}{G}"),
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[], bind_to_life),
    text: "Mill seven cards. Then put a creature card from among them onto the battlefield.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&BIND_TO_LIFE),
    ..oracle::VASTLANDS_SCAVENGER
};

pub static EXTRA: &[&CardDef] = &[&BIND_TO_LIFE];
