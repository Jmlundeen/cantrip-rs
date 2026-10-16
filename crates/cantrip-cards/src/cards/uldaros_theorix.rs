use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_IF_CAST;
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// The total mana value of the spells cast from among the copies.
const BUDGET: u32 = 6;

/// The card types a "nonland card of each card type" can be chosen for (CR 205.2a): every card type but land.
const TYPES: [CardType; 8] = [
    CardType::Kindred,
    CardType::Artifact,
    CardType::Battle,
    CardType::Creature,
    CardType::Enchantment,
    CardType::Instant,
    CardType::Planeswalker,
    CardType::Sorcery,
];

/// The types of `card` among `TYPES`, one bit each.
fn type_bits(q: &dyn Query, card: ObjectId) -> u8 {
    let chars = q.chars(card);
    (0..TYPES.len()).filter(|&i| chars.is(TYPES[i])).fold(0, |bits, i| bits | 1 << i)
}

/// Whether each card can stand for a different card type it has (a matching of cards to types).
fn one_type_each(cards: &[u8], taken: u8) -> bool {
    let Some((&first, rest)) = cards.split_first() else { return true };
    (0..TYPES.len()).any(|i| first & !taken & 1 << i != 0 && one_type_each(rest, taken | 1 << i))
}

/// Whether `card` is a nonland card in the graveyard of the asker, still the card it was.
fn nonland_card_in_your_graveyard(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    targets::card_in_your_graveyard(q, asker, Target::Object(card), &[]) && !q.chars(card).is(CardType::Land)
}

/// One of "up to one target nonland card of each card type from your graveyard" (CR 115.1, 115.3): a nonland card in your
/// graveyard that, with the cards the earlier slots chose, can be a different card type each. A card with several types
/// is one target, chosen for one type only: the same object can't be chosen twice for one instance of the word "target",
/// so an artifact creature card is the artifact or the creature, not both. A land card is not a nonland card whatever else
/// it is (Urza's Saga). The slots are alike, so a slot skipped for want of candidates (the core keeps no place for it)
/// changes nothing; an earlier target that has left the graveyard is not counted, as the rest of the set was legal with it.
fn nonland_card_of_a_type_not_yet_chosen(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(card) = t else { return false };
    if !nonland_card_in_your_graveyard(q, asker, card) || q.chosen_targets().contains(&t) {
        return false;
    }
    let mut bits: Vec<u8> = q
        .chosen_targets()
        .iter()
        .filter_map(|c| c.object())
        .filter(|&c| nonland_card_in_your_graveyard(q, asker, c))
        .map(|c| type_bits(q, c))
        .collect();
    bits.push(type_bits(q, card));
    one_type_each(&bits, 0)
}

/// Up to one target for each of the eight card types a nonland card can have.
const SLOTS: [TargetSpec; TYPES.len()] = [TargetSpec { min: 0, max: 1, filter: nonland_card_of_a_type_not_yet_chosen, label: "Exile up to one target nonland card from your graveyard of a card type none of your other targets has, and copy it" }; TYPES.len()];

/// "Exile up to one target nonland card of each card type from your graveyard. Copy those cards. You may cast any number
/// of spells with total mana value 6 or less from among the copies without paying their mana costs." The targets still
/// legal are exiled together (CR 608.2b, 101.4); each is cast as a copy of the exiled card (CR 707.12), which a permanent
/// spell resolves into as a token (CR 707.10f) and which ceases to exist if it is not cast (CR 704.5e). The spells are
/// chosen one at a time, in the order the controller likes, each within what is left of the six (CR 608.2g); an X in a
/// mana cost is 0 (CR 107.3b), so the mana value counted is the printed one without it.
fn exile_copy_and_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let cards = targets::legal_objects(g, item);
    let mut found = g.exile_all(&cards);
    let mut left = BUDGET;
    loop {
        // A copy is a spell to cast as one face (CR 709.3, 715.3): the half or the Adventure chosen is the spell's, and only it
        // is evaluated to be cast (CR 709.3a, 715.3a), so a copy can be cast when one of its faces fits what is left of the six.
        let fitting = |g: &dyn Game, c: ObjectId| -> Vec<CastWay> {
            g.cast_ways_during_resolution(c).into_iter().filter(|w| w.mana_value <= left).collect()
        };
        let castable: Vec<ObjectId> = found.iter().copied().filter(|&c| !fitting(g, c).is_empty()).collect();
        let chosen = choose::cards_from(g, you, item.object, Purpose::CastCopyFreeWithin(left.min(255) as u8), found.clone(), castable, 1, true)?;
        let Some(&card) = chosen.first() else { return Ok(()) };
        // The face is chosen before the spell is put onto the stack (CR 709.3), asked only when there is a choice.
        let ways = fitting(g, card);
        let Some(way) = choose::cast_way(g, you, card, ways)? else { return Ok(()) };
        let terms = CastTerms { instead: Some(ManaCost::ZERO), copy: true, way: Some(way.way), ..CastTerms::default() };
        // A cast the player gives up is undone (CR 733.1): the copy is not cast and counts for nothing, and they may choose again.
        if g.cast_during_resolution_as(card, terms)? {
            found.retain(|&c| c != card);
            left -= way.mana_value;
        }
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS_IF_CAST, exile_copy_and_cast, "When Uldaros Theorix enters, if you cast him, exile up to one target nonland card of each card type from your graveyard. Copy those cards. You may cast any number of spells with total mana value 6 or less from among the copies without paying their mana costs. (Permanent spells cast this way become tokens.)")
        .targets(&SLOTS)],
    ..oracle::ULDAROS_THEORIX
};
