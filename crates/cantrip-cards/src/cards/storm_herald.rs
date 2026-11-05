use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{choose, delayed, select};
use cantrip_oracle as oracle;

/// "Return any number of Aura cards from your graveyard to the battlefield attached to creatures you control" (CR 303.4f, 303.4i,
/// 614.12): the controller chooses the Auras and their hosts as the ability resolves, not targeting, so an Aura with no
/// creature they control that it could enchant can't be returned (the ruling) and is not offered; each chosen Aura then gets one of
/// the creatures it could enchant, in the order chosen, and they all enter together (CR 603.6a). Each host is judged before any
/// Aura moves, so an Aura never enchants another entering with it. "If those Auras would leave the battlefield, exile them instead
/// of putting them anywhere else" (CR 614.1a) is an effect on exactly those objects. "Exile those Auras at the
/// beginning of your next end step" is a delayed ability about them (CR 603.7, 603.7c).
fn return_auras(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let shown: Vec<ObjectId> = g.state().zone(Zone::graveyard(you)).iter().copied().filter(|&id| g.chars(id).has_subtype(Subtype::Aura)).collect();
    let eligible: Vec<ObjectId> =
        shown.iter().copied().filter(|&aura| creatures.iter().any(|&host| g.could_enter_attached(aura, you, host))).collect();
    let rule = Selection::listed(select::ANY_NUMBER, &shown, eligible).called("Aura");
    let chosen = choose::select(g, you, item.object, Purpose::PutOntoBattlefield, shown, rule)?;
    let mut entering = Vec::new();
    for aura in chosen {
        // The Aura names the question, as the core's own Aura-host choice does, so that with several chosen the player knows which one
        // this creature is for; `Choose` captions it "<Aura>: choose a creature", where `Attach` would read as if the creature were attached.
        let hosts: Vec<ObjectId> = creatures.iter().copied().filter(|&host| g.could_enter_attached(aura, you, host)).collect();
        if let Some(&host) = choose::select(g, you, aura, Purpose::Choose, hosts, select::creature_permanents(select::exactly(1)))?.first() {
            entering.push((aura, you, host));
        }
    }
    let entered: Vec<ObjectId> = g.put_all_onto_battlefield_attached(&entering)?.into_iter().flatten().collect();
    for &aura in &entered {
        g.exile_on_leave(aura);
    }
    if !entered.is_empty() {
        delayed::at_your_next_end_step_about(g, item, 0, &entered);
    }
    Ok(())
}

/// Haste is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, return_auras, "When this creature enters, return any number of Aura cards from your graveyard to the battlefield attached to creatures you control. Exile those Auras at the beginning of your next end step. If those Auras would leave the battlefield, exile them instead of putting them anywhere else.")],
    delayed: &[delayed_trigger(delayed::exile_subject, "Exile those Auras.")],
    ..oracle::STORM_HERALD
};
