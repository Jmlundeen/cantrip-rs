use cantrip_api::*;
use cantrip_mechanics::effects::may_return_this_card_to_battlefield;
use cantrip_oracle as oracle;

/// "Whenever you cast a spell, if it's the second creature spell you cast this turn": this turn's history says how
/// each spell was cast, this one included (CR 603.4: a fact about the past).
fn second_creature_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    let owner = q.obj(source).owner;
    if *player != owner || !q.history().cast_of(*spell).is_some_and(|c| c.is(CardType::Creature)) {
        return None;
    }
    let creatures = q.history().casts_by(owner).filter(|c| c.is(CardType::Creature)).count();
    if creatures == 2 { Fired::PLAIN } else { None }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::SPELL_CAST, second_creature_spell), may_return_this_card_to_battlefield, "Whenever you cast a spell, if it's the second creature spell you cast this turn, you may return this card from your graveyard to the battlefield.")
        // It returns this card from the graveyard, so it functions only there (CR 113.6m).
        .functions_in(IN_A_GRAVEYARD)],
    ..oracle::VENGEVINE
};
