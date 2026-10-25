use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::cloak::cloak_top_of_library;
use cantrip_mechanics::effects::return_this_to_hand;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Cloak the top card of your library, then attach this Equipment to it" (CR 701.58a, 701.3b): with an empty library
/// nothing is cloaked and the Equipment stays unattached.
fn cloak_and_attach(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = cloak_top_of_library(g, item.controller, item.controller)? {
        g.attach(item.object, permanent);
    }
    Ok(())
}

/// "Equipped creature ... can't be blocked" (CR 509.1b): no "has", so a rule of the Equipment's, not an ability the creature
/// gains (CR 113.1a). A creature that loses all its abilities is still under it (CR 613.1f), and it is no ability of the
/// creature's for anything that asks ("creatures with no abilities").
fn equipped_creature_cant_be_blocked(q: &dyn Query, source: ObjectId, _: ObjectId, attacker: ObjectId) -> bool {
    q.obj(source).attached_to != Some(attacker)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, cloak_and_attach, "When this Equipment enters, cloak the top card of your library, then attach this Equipment to it."),
        ability(cost("{1}{U}"), return_this_to_hand, "{1}{U}: Return this Equipment to its owner's hand."),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<1, 0>, "Equipped creature gets +1/+0."),
    ],
    block_rules: &[equipped_creature_cant_be_blocked],
    ..oracle::CRYPTIC_COAT
};
