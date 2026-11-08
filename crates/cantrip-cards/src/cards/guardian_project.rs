use cantrip_api::*;
use cantrip_mechanics::same_name;
use cantrip_mechanics::triggers::{nontoken_you_control_enters, CREATURE_TYPE};
use cantrip_oracle as oracle;

/// "Whenever a nontoken creature you control enters, if it doesn't have the same name as another creature you control or a
/// creature card in your graveyard, draw a card." (CR 603.6a, 201.2, 603.4): the entering creature is the ability's subject;
/// "another" leaves the creature itself out even though its new object is on the battlefield already, and a name is shared by
/// any other creature its controller controls or any creature card in their graveyard. The "if" is an intervening condition,
/// asked by the trigger matcher as the creature enters and again as the ability resolves.
fn shares_its_name(q: &dyn Query, player: PlayerId, id: ObjectId) -> bool {
    let other_creatures = q.permanents_of(player).into_iter()
        .any(|other| other != id && q.chars(other).is(CardType::Creature) && same_name(q, id, other));
    let graveyard_cards = q.state().zone(Zone::graveyard(player)).iter()
        .any(|&card| q.chars(card).is(CardType::Creature) && same_name(q, id, card));
    other_creatures || graveyard_cards
}

/// "Whenever a nontoken creature you control enters" (CR 603.6a) "if it doesn't have the same name as another creature you
/// control or a creature card in your graveyard" (CR 603.4): the entering creature, when no other creature the controller
/// controls and no creature card in their graveyard already has its name.
fn a_nontoken_creature_of_a_new_name_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let fired = nontoken_you_control_enters::<CREATURE_TYPE>(q, me, event)?;
    match fired.subject {
        Target::Object(id) if !shares_its_name(q, me.controller, id) => Some(fired),
        _ => None,
    }
}

const NONTOKEN_CREATURE_OF_A_NEW_NAME_ENTERS: Trigger =
    when_asked(EventMask::ZONE_CHANGE, a_nontoken_creature_of_a_new_name_enters);

/// "draw a card" (CR 121.1): one card, to the ability's controller, if the name is still unshared as the ability resolves
/// (CR 603.4, 608.2c).
fn draw_for_the_new_name(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.subject
        && !shares_its_name(g, item.controller, id)
    {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        NONTOKEN_CREATURE_OF_A_NEW_NAME_ENTERS,
        draw_for_the_new_name,
        "Whenever a nontoken creature you control enters, if it doesn't have the same name as another creature you control or a creature card in your graveyard, draw a card.",
    )],
    ..oracle::GUARDIAN_PROJECT
};
