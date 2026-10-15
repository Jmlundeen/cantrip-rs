use cantrip_api::*;
use cantrip_mechanics::copy::{create_token_copy_of_it, not_a_token};
use cantrip_mechanics::triggers::this_dies;
use cantrip_oracle as oracle;

/// "Whenever this creature or another creature you control with power 4 or greater enters" (CR 603.2): its own entry,
/// or another creature of its controller's whose power, as it entered, is 4 or more.
fn enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && (*new == source
                    || (q.controller(*new) == q.controller(source)
                        && q.chars(*new).is(CardType::Creature)
                        && q.chars(*new).power.is_some_and(|p| p >= 4))) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn gain_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    g.draw(item.controller, 1);
    Ok(())
}

/// Create a token that's a copy of it, except it's an artifact in addition to its other types (CR 707.9b).
fn create_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_token_copy_of_it(g, item, |_, values| values.with_type(CardType::Artifact))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature or another creature you control with power 4 or greater enters, you gain 3 life and draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: enters,
            effect: gain_and_draw,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature dies, if it's not a token, create a token that's a copy of it, except it's an artifact in addition to its other types.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_dies,
            look_back: true,
            condition: Some(not_a_token),
            effect: create_copy,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::VAULTBORN_TYRANT
};
