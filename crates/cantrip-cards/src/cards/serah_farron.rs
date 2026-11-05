use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::statics::{plus_pt, your_creatures};
use cantrip_mechanics::{cost_reduction, triggers};
use cantrip_oracle as oracle;

const TRANSFORM: Question = Question::new("TransformSerahFarron", "transform Serah Farron?", "It becomes Crystallized Serah", "Transform", "Don't transform");

fn legendary_creature_spell(chars: &Chars) -> bool {
    chars.is(CardType::Creature) && chars.supertypes.legendary
}

/// "The first legendary creature spell you cast each turn costs {2} less to cast" (CR 601.2f, 118.7), on both faces: generic mana
/// only, and a legendary creature spell cast earlier this turn is the first, whether or not this was on the battlefield for it.
fn first_legendary_creature_spell_costs_two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    cost_reduction::first_spell_you_cast_each_turn(q, source, player, costed, 2, legendary_creature_spell)
}

/// The beginning-of-combat ability's intervening "if" (CR 603.4): two or more legendary creatures its controller controls besides
/// Serah Farron herself, checked as combat begins and again as the ability resolves.
fn two_other_legendary_creatures(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().filter(|&id| id != asker.source && legendary_creature_spell(&q.chars(id))).count() >= 2
}

/// "You may transform Serah Farron." The ability's source, which does nothing if it is no longer that object (CR 400.7) or has
/// transformed since the ability went on the stack (CR 701.27f).
fn may_transform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may(g, item.controller, item.object, Purpose::Ask(&TRANSFORM))? {
        g.transform_by(item.object, item)?;
    }
    Ok(())
}

/// "Legendary creatures you control" (CR 613.1f): the creatures its controller controls that the earlier layers made legendary.
fn your_legendary_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.supertypes.legendary && your_creatures(q, source, affected, so_far)
}

pub static BACK: CardDef = CardDef {
    cost_changes: &[first_legendary_creature_spell_costs_two_less],
    statics: &[static_ability(Layer::PtModify, your_legendary_creatures, plus_pt::<2, 2>, "Legendary creatures you control get +2/+2.")],
    ..oracle::CRYSTALLIZED_SERAH
};

pub static CARD: CardDef = CardDef {
    cost_changes: &[first_legendary_creature_spell_costs_two_less],
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        may_transform,
        "At the beginning of combat on your turn, if you control two or more other legendary creatures, you may transform Serah Farron.",
    )
    .condition(two_other_legendary_creatures)],
    back: Some(&BACK),
    ..oracle::SERAH_FARRON
};
