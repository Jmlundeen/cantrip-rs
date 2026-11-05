use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on up to one target creature. Draw a card." (CR 122.1a, 121.1): the counter goes on if the target is still
/// legal, then the card is drawn whether or not a target was chosen (CR 608.2b).
fn counter_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_target(g, item)?;
    effects::draw(g, item, 1)
}

/// The Omen (CR 720): Dusk Sight, cast from the hand as an Omen sorcery for {1}{B}; as it resolves it is shuffled into its owner's
/// library (CR 720.3d, the core's).
pub static DUSK_SIGHT: CardDef = CardDef {
    spell: spell(&[targets::CREATURE.between(0, 1)], counter_then_draw),
    ..oracle::DUSK_SIGHT
};

/// "Exile up to two target cards from a single graveyard" (CR 608.2b): the cards still there are exiled together. Unlicensed Hearse
/// prints the same targets.
fn exile_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<ObjectId> = (0..2).filter_map(|slot| targets::object(g, item, slot)).collect();
    g.exile_all(&cards);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, exile_cards, "When this creature enters, exile up to two target cards from a single graveyard.").targets(&[
        targets::CARD_IN_A_GRAVEYARD.optional().labelled("First card to exile (up to two, from a single graveyard)"),
        targets::ANOTHER_CARD_IN_THE_SAME_GRAVEYARD.optional().labelled("Second card to exile, from the same graveyard"),
    ])],
    omen: Some(&DUSK_SIGHT),
    ..oracle::FERAL_DEATHGORGER
};

pub static EXTRA: &[&CardDef] = &[&DUSK_SIGHT];
