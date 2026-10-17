use cantrip_api::*;
use cantrip_mechanics::ripple;
use cantrip_oracle as oracle;

/// "Spells you cast" (CR 601.2i): a spell on the stack that was cast, not a copy of one (CR 707.10a), and that its
/// controller now is the Stone's. (A spell an opponent cast and you gained control of would also count: a control
/// change of a spell on the stack nothing can make.)
fn spell_you_cast(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    cantrip_mechanics::statics::spell_you_cast(view, source, affected)
}

pub static CARD: CardDef = CardDef {
    delayed: &[ripple::trigger::<4>("Ripple 4 (When you cast this spell, you may reveal the top four cards of your library. You may cast spells with the same name as this spell from among the revealed cards without paying their mana costs. Put the rest on the bottom of your library.)")],
    statics: &[static_ability(Layer::Ability, spell_you_cast, cantrip_mechanics::statics::grant_delayed_ability, "Spells you cast have ripple 4.")
        .everywhere()],
    ..oracle::THRUMMING_STONE
};
