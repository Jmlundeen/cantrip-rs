use cantrip_api::*;
use cantrip_mechanics::{choose_color, crew::crew, damage_target, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this Vehicle enters, add four mana of any one color. Spend this mana only to activate abilities." A triggered
/// ability that adds mana is no mana ability (CR 605.1b): it resolves from the stack, the color chosen then (CR 106.5,
/// 106.6). The four may go to one ability or to several (its ruling).
fn four_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let color = choose_color(g, item.controller, item.object)?;
    g.add_restricted_mana(item.controller, color.mana(), 4, ManaRestriction::ABILITIES);
    Ok(())
}

/// "This Vehicle deals X damage to any target. Put a +1/+1 counter on this Vehicle." Nothing at all if the target is no
/// longer legal (CR 608.2b: the core doesn't resolve it); the counter only on the Vehicle that is still on the
/// battlefield (CR 400.7).
fn x_damage_and_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, item.x);
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, four_mana, "When this Vehicle enters, add four mana of any one color. Spend this mana only to activate abilities."),
        ability(cost("{X}{2}{R}"), x_damage_and_counter, "Exhaust \u{2014} {X}{2}{R}: This Vehicle deals X damage to any target. Put a +1/+1 counter on this Vehicle. (Activate each exhaust ability only once.)")
            .targets(&[targets::ANY])
            .exhaust(),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::BOOMMOBILE
};
