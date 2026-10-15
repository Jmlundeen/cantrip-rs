//! Cipher (CR 702.99): "If this spell is represented by a card, you may exile this card encoded on a creature you control"
//! and "For as long as this card is encoded on that creature, that creature has 'Whenever this creature deals combat damage
//! to a player, you may copy the encoded card and you may cast the copy without paying its mana cost.'" Trait Doctoring
//! prints it.
//!
//! A card with cipher calls `encode` last in its spell's effect, lists `STATIC` among its `statics` and `TRIGGER` first in
//! its `delayed` abilities: the creature it is encoded on has that ability.

use crate::choose;
use cantrip_api::*;

/// "You may exile this card encoded on a creature you control" (CR 702.99a), the last thing the spell does as it resolves.
/// A copy of a spell is not represented by a card and is not encoded. The card is exiled linked to the creature (CR
/// 702.99b); it stays encoded while it stays in exile and the creature stays on the battlefield (CR 702.99c).
pub fn encode(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spell = item.object;
    if g.obj(spell).card_copy || !is_live(g, spell) || !g.obj(spell).zone.is(ZoneKind::Stack) {
        return Ok(());
    }
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    if let Some(&creature) = choose::cards(g, item.controller, spell, Purpose::Encode, creatures, 1, true)?.first() {
        g.encode(spell, creature);
    }
    Ok(())
}

/// The creature this card is encoded on (CR 702.99b, 702.99c): the one cipher exiled it encoded on, while the card is
/// still in exile and the creature still the permanent it was. A card exiled linked to a creature any other way is not
/// encoded.
fn encoded_on(state: &GameState, card: ObjectId) -> Option<ObjectId> {
    let obj = state.obj(card);
    let link = obj.exiled.filter(|l| l.encoded)?;
    let creature = state.obj(link.by);
    (obj.zone.is(ZoneKind::Exile) && obj.became.is_none() && creature.became.is_none() && creature.zone.is(ZoneKind::Battlefield))
        .then_some(link.by)
}

fn affects(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    encoded_on(view.state(), source) == Some(affected)
}

/// "That creature has" the ability (CR 702.99a, 613.1f): the card's delayed ability 0, an ability of the creature, so
/// its controller controls it, and it is lost with the creature's abilities.
fn grant(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let card = view.state().obj(source).card;
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub const STATIC: StaticDef = StaticDef {
    text: "For as long as this card is encoded on that creature, that creature has \"Whenever this creature deals combat damage to a player, you may copy the encoded card and you may cast the copy without paying its mana cost.\"",
    layer: Layer::Ability,
    affects,
    apply: grant,
    from: ZoneKind::Exile,
    ..StaticDef::BLANK
};

/// "You may copy the encoded card and you may cast the copy without paying its mana cost" (CR 702.99a, 707.12): the card
/// encoded on this creature that granted this ability, if it still is: a creature that has left the battlefield since
/// the ability triggered has no card encoded on it any more (CR 702.99c). The card stays encoded.
fn cast_a_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = item.object;
    let encoded = g
        .state()
        .seats()
        .flat_map(|p| g.state().zone(Zone::exile(p)).iter().copied())
        .find(|&c| g.obj(c).card == item.ability.card && encoded_on(g.state(), c) == Some(creature));
    let Some(card) = encoded else { return Ok(()) };
    if !g.could_cast_during_resolution(card) {
        return Ok(());
    }
    if g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::Cast })? == Answer::Yes {
        g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), copy: true, ..CastTerms::default() })?;
    }
    Ok(())
}

pub const TRIGGER: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "Whenever this creature deals combat damage to a player, you may copy the encoded card and you may cast the copy without paying its mana cost.",
    listens: EventMask::DAMAGE,
    matches: crate::triggers::deals_combat_damage_to_player,
    effect: cast_a_copy,
    ..TriggeredDef::BLANK
});
