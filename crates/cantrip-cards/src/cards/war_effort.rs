use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::{delayed, mobilize, statics, triggers};
use cantrip_oracle as oracle;

/// "Create a 1/1 red Warrior creature token that's tapped and attacking" (CR 508.4): its controller chooses what it
/// attacks, asked only where there is a choice. "Sacrifice it at the beginning of the next end step" (CR 603.7c) is the
/// card's delayed ability 0, about exactly the token made.
fn warrior(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let warrior = g.card_id(&mobilize::WARRIOR);
    let created = create_attacking_of_your_choice(g, item.controller, item.object, warrior, 1, true)?;
    delayed::at_next_end_step_about(g, item, 0, &created);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, statics::your_creatures, statics::plus_pt::<1, 0>, "Creatures you control get +1/+0.")],
    abilities: &[triggered(
        triggers::YOU_ATTACK,
        warrior,
        "Whenever you attack, create a 1/1 red Warrior creature token that's tapped and attacking. Sacrifice it at the beginning of the next end step.",
    )],
    delayed: &[delayed_trigger(delayed::sacrifice_created, "Sacrifice it at the beginning of the next end step.")],
    ..oracle::WAR_EFFORT
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
