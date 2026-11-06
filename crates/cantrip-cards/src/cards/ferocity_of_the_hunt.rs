use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::statics::{have_deathtouch, plus_pt};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return that card to the battlefield tapped under its owner's control" (CR 110.2a): the card as it is in the graveyard.
fn return_host(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield(card, owner, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(cantrip_mechanics::triggers::ATTACHED_CREATURE_DIES_ABOUT_THE_CARD, return_host, "When enchanted creature dies, return that card to the battlefield tapped under its owner's control.")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<1, 0>, "Enchanted creature gets +1/+0."),
        static_ability(Layer::Ability, attached_object, have_deathtouch, "Enchanted creature has deathtouch."),
    ],
    ..oracle::FEROCITY_OF_THE_HUNT
};
