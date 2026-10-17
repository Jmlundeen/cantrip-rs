use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

fn target_instant_or_sorcery_spell(q: &dyn Query, _: &Asker, target: Target) -> bool {
    matches!(target, Target::Object(id) if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
        && { let c = q.chars(id); c.is(CardType::Instant) || c.is(CardType::Sorcery) })
}

fn test(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(spell);
    let names = library::object_names(g, spell);
    g.counter(spell);
    library::exile_same_name_from_graveyard_hand_and_library(g, item.controller, controller, item.object, &names)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(target_instant_or_sorcery_spell)], test),
    ..oracle::TEST_OF_TALENTS
};
