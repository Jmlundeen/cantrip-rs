use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Exhaust \u{2014} Waterbend {3}: This Vehicle becomes an artifact creature. Put three +1/+1 counters on it." With no duration
/// said, it stays a creature for as long as it is on the battlefield (CR 611.2a), an artifact too (it already is one) and still a
/// Vehicle; its 0/0 and the three counters make it a 3/3. Done to the permanent that paid the cost, so a Vehicle that has left the
/// battlefield by the time the ability resolves is a new object and gets nothing (CR 400.7).
fn become_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::AddTypes(TypeSet::of(&[CardType::Artifact, CardType::Creature])), Duration::Indefinite);
    g.add_counters(item.object, CounterKind::PlusOne, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::return_first_target_to_hand, "When this Vehicle enters, return up to one other target nonland permanent to its owner's hand.")
            .targets(&[up_to_one(targets::other_nonland_permanent_filter)]),
        ability(cost("{3}"), become_creature, "Exhaust \u{2014} Waterbend {3}: This Vehicle becomes an artifact creature. Put three +1/+1 counters on it. (Activate each exhaust ability only once.)")
            .waterbend()
            .exhaust(),
    ],
    ..oracle::INVASION_SUBMERSIBLE
};
