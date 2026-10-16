use cantrip_api::*;
use cantrip_mechanics::ripple;
use cantrip_oracle as oracle;

/// "Spells you cast" (CR 601.2i): a spell on the stack that was cast, not a copy of one (CR 707.10a), and that its
/// controller now is the Stone's. (A spell an opponent cast and you gained control of would also count: a control
/// change of a spell on the stack nothing can make.)
fn spell_you_cast(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let s = view.state();
    let obj = s.obj(affected);
    obj.zone.is(ZoneKind::Stack)
        && !obj.card_copy
        && obj.cast_from.is_some()
        && s.stack.iter().any(|i| i.kind == StackKind::Spell && i.object == affected && i.controller == s.obj(source).controller)
}

/// Layer 6 (CR 613.1f): ripple's triggered ability, the Stone's delayed ability 0, on each such spell. Each Stone grants
/// its own instance, and each triggers separately (CR 702.60b).
fn grant_ripple(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    delayed: &[ripple::trigger::<4>("Ripple 4 (When you cast this spell, you may reveal the top four cards of your library. You may cast spells with the same name as this spell from among the revealed cards without paying their mana costs. Put the rest on the bottom of your library.)")],
    statics: &[static_ability(Layer::Ability, spell_you_cast, grant_ripple, "Spells you cast have ripple 4.")
        .everywhere()],
    ..oracle::THRUMMING_STONE
};
