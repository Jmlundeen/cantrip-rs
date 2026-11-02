use cantrip_api::*;
use cantrip_mechanics::statics::{have_menace, have_trample, plus_pt};
use cantrip_mechanics::{attach, effects, plot, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_DIES,
            |g, item| effects::draw(g, item, 1),
            "When this Aura is put into a graveyard from the battlefield, draw a card.",
        ),
        plot::plot(mana("{R}"), "Plot {R}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attach::attached_object, plus_pt::<1, 1>, "Enchanted creature gets +1/+1 and has menace and trample."),
        static_ability(Layer::Ability, attach::attached_object, have_menace, "Enchanted creature gets +1/+1 and has menace and trample."),
        static_ability(Layer::Ability, attach::attached_object, have_trample, "Enchanted creature gets +1/+1 and has menace and trample."),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::DEMONIC_RUCKUS
};
