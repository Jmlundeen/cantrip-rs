use cantrip_api::*;
use cantrip_mechanics::{craft, effects};
use cantrip_oracle as oracle;

/// "Red instant and/or sorcery cards": named as cards, so only cards in your graveyard (CR 702.167b), red read through the
/// Stalactite's words (CR 612).
fn red_instant_or_sorcery_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    let c = q.chars(id);
    q.obj(id).zone.is(ZoneKind::Graveyard)
        && c.colors.has(q.words(asker.source).color(Color::Red))
        && (c.is(CardType::Instant) || c.is(CardType::Sorcery))
}

static FOUR_OR_MORE: Materials = craft::materials(4, true, red_instant_or_sorcery_card, "red instant and/or sorcery card");

/// "Choose an exiled card used to craft this artifact at random. You may cast that card without paying its mana cost."
/// (CR 702.167c, 608.2g) The card is chosen among those still in exile, its timing waived; the controller is asked
/// whether to cast it, and nothing is asked when it could not be cast at all.
fn cast_one_at_random(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let crafted = craft::exiled_cards_used_to_craft(g, item.object);
    if crafted.is_empty() {
        return Ok(());
    }
    let card = crafted[g.choose_at_random(crafted.len())];
    if g.could_cast_during_resolution(card) && effects::ask_may(g, item.controller, card, Purpose::CastFree)? {
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[ability(cost("{1}{R}, {T}"), cast_one_at_random, "{1}{R}, {T}: Choose an exiled card used to craft this artifact at random. You may cast that card without paying its mana cost.")],
    ..oracle::COSMIUM_CATALYST
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R).only_for(ManaRestriction::INSTANT_OR_SORCERY_SPELLS), "{T}: Add {R}. Spend this mana only to cast an instant or sorcery spell."),
        craft::ability(mana("{3}{R}{R}"), &FOUR_OR_MORE, "Craft with four or more red instant and/or sorcery cards {3}{R}{R}"),
    ],
    back: Some(&BACK),
    ..oracle::ORE_RICH_STALACTITE
};
