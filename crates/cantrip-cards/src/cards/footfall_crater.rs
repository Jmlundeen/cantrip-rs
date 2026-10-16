use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets, triggers::cycling};
use cantrip_oracle as oracle;

/// Target creature gains trample and haste until end of turn: the granted ability's source is the land (CR 113.7).
fn trample_and_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(
            item.object,
            creature,
            EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample, Keyword::Haste])),
            Duration::EndOfTurn,
        );
    }
    Ok(())
}

/// Enchanted land has "{T}: Target creature gains trample and haste until end of turn": the Aura's own delayed ability
/// 0, an activated ability the land has (layer 6, CR 613.1f) for as long as the Aura is attached to it.
fn grant_ability(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::LAND),
    abilities: &[cycling(mana("{1}"), "Cycling {1} ({1}, Discard this card: Draw a card.)")],
    delayed: &[ability(TAP, trample_and_haste, "{T}: Target creature gains trample and haste until end of turn.")
        .targets(&[targets::CREATURE])],
    statics: &[static_ability(Layer::Ability, attached_object, grant_ability, "Enchanted land has \"{T}: Target creature gains trample and haste until end of turn.\"")],
    ..oracle::FOOTFALL_CRATER
};
