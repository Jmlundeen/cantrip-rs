use cantrip_api::*;
use cantrip_mechanics::history::gained_life_this_turn;
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_ENTERS, YOUR_END_STEP};
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Whenever this token attacks, you gain 1 life."
fn gain_a_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 1)
}

/// The 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life."
pub static PEST: CardDef = CardDef {
    name: "Pest",
    has_cost: false,
    colors: Some(ColorSet(0b10100)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Pest]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[triggered(THIS_ATTACKS, gain_a_life, "Whenever this token attacks, you gain 1 life.")],
    text: "Whenever this token attacks, you gain 1 life.",
    ..CardDef::BLANK
};

/// "When Moseo enters, create a 1/1 black and green Pest creature token with 'Whenever this token attacks, you gain 1 life.'"
fn create_a_pest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &PEST, 1)?;
    Ok(())
}

/// "If you gained life this turn" (CR 603.4): the intervening "if" of the infusion ability.
fn gained_life(q: &dyn Query, asker: &Asker) -> bool {
    gained_life_this_turn(q, asker.controller)
}

/// "Target creature card with mana value X or less from your graveyard, where X is the amount of life you gained this turn"
/// (CR 202.3): the life gained is read from the history each time the target is judged, as it is chosen and again as the
/// ability resolves (CR 608.2b).
fn creature_card_with_mana_value_at_most_life_gained(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let gained = q.history().life_gained(asker.controller);
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], gained)
}

/// "Return up to one target creature card ... from your graveyard to the battlefield."
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_a_pest, "When Moseo enters, create a 1/1 black and green Pest creature token with \"Whenever this token attacks, you gain 1 life.\""),
        triggered(YOUR_END_STEP, reanimate, "Infusion \u{2014} At the beginning of your end step, if you gained life this turn, return up to one target creature card with mana value X or less from your graveyard to the battlefield, where X is the amount of life you gained this turn.")
            .targets(&[up_to_one(creature_card_with_mana_value_at_most_life_gained)])
            .condition(gained_life),
    ],
    ..oracle::MOSEO_VEINS_NEW_DEAN
};

pub static EXTRA: &[&CardDef] = &[&PEST];
