use cantrip_api::*;
use cantrip_mechanics::statics::have_haste;
use cantrip_oracle as oracle;

/// Creatures its owner controls, while the owner controls a Mountain. The source functions only from the graveyard
/// (CR 113.6b), which `from` says.
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let state = q.state();
    let you = state.obj(source).owner;
    if !so_far.is(CardType::Creature) || state.obj(affected).controller != you {
        return false;
    }
    state.battlefield().iter().any(|&id| state.obj(id).controller == you && q.chars(id).has_subtype(q.words(source).subtype(Subtype::Mountain)))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, affects, have_haste, "As long as this card is in your graveyard and you control a Mountain, creatures you control have haste.")
        .from(ZoneKind::Graveyard)],
    ..oracle::ANGER
};
