use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever you cast a spell": a spell, not an activated ability, cast by this permanent's controller.
fn you_cast_a_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "Untap all nonland permanents you control" (CR 701.26b): everything its controller then controls that is not a
/// land, with no choice and no count.
fn untap_nonland(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let nonland: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| !g.chars(id).is(CardType::Land)).collect();
    for id in nonland {
        g.untap(id);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::SPELL_CAST, you_cast_a_spell), untap_nonland, "Whenever you cast a spell, untap all nonland permanents you control.")],
    ..oracle::PARADOX_ENGINE
};
