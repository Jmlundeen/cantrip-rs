use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::history::first_combat_phase_of_the_turn;
use cantrip_mechanics::statics::have_double_strike;
use cantrip_mechanics::triggers::ATTACHED_CREATURE_ATTACKS_SUBJECT;
use cantrip_oracle as oracle;

/// Untap it, the equipped creature that attacked (CR 701.26a). After this
/// phase, there is an additional combat phase (CR 500.8).
fn untap_it_and_add_a_combat_phase(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        g.untap(creature);
    }
    g.add_combat_phase();
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            ATTACHED_CREATURE_ATTACKS_SUBJECT,
            untap_it_and_add_a_combat_phase,
            "Whenever equipped creature attacks, if it's the first combat phase of the turn, untap it. After this phase, there is an additional combat phase.",
        )
        .condition(first_combat_phase_of_the_turn),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[static_ability(Layer::Ability, attached_object, have_double_strike, "Equipped creature has double strike.")],
    ..oracle::GENJI_GLOVE
};
