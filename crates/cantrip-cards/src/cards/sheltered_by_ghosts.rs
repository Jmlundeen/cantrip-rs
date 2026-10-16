use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, exile_target_until_leaves, targets, triggers::{THIS_ENTERS, ward}};
use cantrip_oracle as oracle;

/// "Exile target nonland permanent an opponent controls until this Aura leaves the battlefield" (CR 610.3).
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

fn plus_one_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
}

fn lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink);
}

/// The enchanted creature has ward {2} (CR 702.21a): the Aura's own delayed ability 0, a triggered ability the creature
/// controls and that is gone the moment the Aura stops enchanting it.
fn grant_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    abilities: &[triggered(THIS_ENTERS, exile_until_it_leaves, "When Sheltered by Ghosts enters, exile target nonland permanent an opponent controls until Sheltered by Ghosts leaves the battlefield.")
        .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS])],
    delayed: &[ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_one_power, "Enchanted creature gets +1/+0."),
        static_ability(Layer::Ability, attached_object, lifelink, "Enchanted creature has lifelink."),
        static_ability(Layer::Ability, attached_object, grant_ward, "Enchanted creature has ward {2}."),
    ],
    ..oracle::SHELTERED_BY_GHOSTS
};
