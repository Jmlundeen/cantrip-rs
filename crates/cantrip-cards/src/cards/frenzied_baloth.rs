use cantrip_api::*;
use cantrip_mechanics::statics::cant_be_countered;
use cantrip_oracle as oracle;

/// "Creature spells you control": a creature spell on the stack (an ability on the stack has no card types) controlled by
/// the player who controls this permanent, judged on the characteristics the earlier layers gave it.
fn your_creature_spells(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let o = q.state().obj(affected);
    o.zone.kind() == ZoneKind::Stack && so_far.is(CardType::Creature) && o.controller == q.state().obj(source).controller
}

/// "Combat damage can't be prevented" (CR 615.12): the combat damage of every source, whoever controls it.
fn combat_damage(_: &dyn Query, _: &Asker, _: ObjectId, combat: bool) -> bool {
    combat
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, your_creature_spells, cant_be_countered, "Creature spells you control can't be countered.").everywhere()],
    unpreventable_damage: &[combat_damage],
    ..oracle::FRENZIED_BALOTH
};
