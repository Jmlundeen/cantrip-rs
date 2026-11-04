use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::{have_flying, plus_pt};
use cantrip_mechanics::{tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Create a 2/2 colorless Robot artifact creature token and attach this Equipment to it" (CR 111.1, 701.3a): the Equipment attaches
/// only if the token was made and is still there; attaching does nothing for an Equipment that has left (CR 701.3b, 400.7).
/// With a token doubler there are two tokens: it goes onto the first.
fn create_a_robot_and_attach_this_to_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &tokens::ROBOT_2_2, 1)?;
    if let Some(&robot) = made.first()
        && is_live(g, robot)
        && g.is_on_battlefield(robot)
    {
        g.attach(item.object, robot);
    }
    Ok(())
}

/// Flying is granted by the static ability below.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            create_a_robot_and_attach_this_to_it,
            "When this Equipment enters, create a 2/2 colorless Robot artifact creature token and attach this Equipment to it.",
        ),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<1, 2>, "Equipped creature gets +1/+2."),
        static_ability(Layer::Ability, attached_object, have_flying, "Equipped creature has flying."),
    ],
    ..oracle::AUXILIARY_BOOSTERS
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_2_2];
