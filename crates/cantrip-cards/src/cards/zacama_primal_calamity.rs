use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "When Zacama enters, if you cast it, untap all lands you control." (CR 603.4): the trigger is the shared "if it was cast"; "you"
/// is asked again as the ability resolves, since a copy of the trigger is controlled by a player who did not cast Zacama. The spell
/// Zacama entered as is the object of the resolution that put it onto the battlefield.
fn if_you_cast_it_untap_all_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let q: &dyn Query = g;
    let permanent = item.object;
    let history = q.history();
    let cast_by_you = history
        .all()
        .iter()
        .rev()
        .find_map(|r| match r.event {
            Event::ZoneChange { old, new, reason: ZoneReason::Resolve, .. } if new == permanent => Some(old),
            _ => None,
        })
        .and_then(|spell| history.cast_of(spell))
        .is_some_and(|cast| cast.player == item.controller);
    if !cast_by_you {
        return Ok(());
    }
    effects::untap_all_lands_you_control(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS_IF_CAST, if_you_cast_it_untap_all_lands, "When Zacama enters, if you cast it, untap all lands you control."),
        ability(cost("{2}{R}"), effects::damage_first_target::<3>, "{2}{R}: Zacama deals 3 damage to target creature.").targets(&[targets::CREATURE]),
        ability(cost("{2}{G}"), effects::destroy_first_target, "{2}{G}: Destroy target artifact or enchantment.")
            .targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
        ability(cost("{2}{W}"), |g, item| effects::gain_life(g, item, 3), "{2}{W}: You gain 3 life."),
    ],
    ..oracle::ZACAMA_PRIMAL_CALAMITY
};
