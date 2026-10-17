use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, statics::plus_pt, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this Aura enters, enchanted creature fights up to one target creature an opponent controls" (CR 701.14). The
/// enchanted creature is the one the Aura is attached to as the ability resolves, or, if the Aura has left the battlefield
/// by then, the one it was attached to as it left (CR 608.2h). The fight itself does nothing unless both are creatures on
/// the battlefield (CR 701.14b); with no target chosen, or the target gone, there is nothing to fight.
fn fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(prey) = targets::object(g, item, 0) else { return Ok(()) };
    if let Some(enchanted) = g.obj(item.object).attached_to {
        g.fight(enchanted, prey);
    }
    Ok(())
}

/// "Enchanted creature ... can't be blocked by more than one creature" (CR 509.1b, layer 6).
fn one_blocker_at_most(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::CantBeBlockedByMoreThanOne);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    abilities: &[triggered(
        THIS_ENTERS,
        fight,
        "When this Aura enters, enchanted creature fights up to one target creature an opponent controls. (Each deals damage equal to its power to the other.)",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS.optional()])],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<0, 2>, "Enchanted creature gets +0/+2."),
        static_ability(Layer::Ability, attached_object, one_blocker_at_most, "Enchanted creature can't be blocked by more than one creature."),
    ],
    ..oracle::MELTSTRIDERS_RESOLVE
};
