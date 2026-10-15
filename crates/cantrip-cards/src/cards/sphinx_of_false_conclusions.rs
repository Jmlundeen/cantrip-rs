use cantrip_api::*;
use cantrip_mechanics::{effects::loot, triggers::{this_attacks, this_dies}};
use cantrip_oracle as oracle;

/// "If it isn't a token" (CR 603.4): checked as the creature dies and again as the ability resolves, on the creature as
/// it last existed on the battlefield.
fn not_a_token(q: &dyn Query, asker: &Asker) -> bool {
    !q.obj(asker.source).token
}

/// "Create a token that's a copy of it" (CR 707.2, 111.1): the copiable values of the creature as it last existed.
fn create_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let values = g.copiable_values(item.object);
    g.create_token_copies_of(values, item.controller, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature attacks, draw a card, then discard a card.",
            listens: EventMask::ATTACKS,
            matches: this_attacks,
            effect: loot,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature dies, if it isn't a token, create a token that's a copy of it.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_dies,
            look_back: true,
            condition: Some(not_a_token),
            effect: create_copy,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SPHINX_OF_FALSE_CONCLUSIONS
};
