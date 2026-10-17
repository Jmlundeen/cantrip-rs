//! Library recipes (CR 401, 701.23): searching, looking at the top, exiling it to play.

use crate::{search_library, top_of_library};
use cantrip_api::*;

/// What a library search looks for (CR 701.23): how many cards, whether finding fewer is allowed, whether the cards
/// found are revealed. `Find::ONE` is "a [quality] card", which a player may fail to find (CR 701.23b); `Find::ANY` is
/// "a card" with nothing to fail at, so the searcher must take one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Find {
    pub count: u8,
    pub optional: bool,
    pub reveal: bool,
}

impl Find {
    /// "Search your library for a [quality] card": one card, which may not be found.
    pub const ONE: Find = Find { count: 1, optional: true, reveal: false };
    /// "Search your library for a card": one card, taken if there is any.
    pub const ANY: Find = Find { count: 1, optional: false, reveal: false };
    /// The cards found are revealed ("reveal it").
    pub const fn revealed(self) -> Find {
        Find { reveal: true, ..self }
    }
    /// Up to `count` cards instead of one.
    pub const fn up_to(self, count: u8) -> Find {
        Find { count, ..self }
    }
}

fn find(g: &mut dyn Game, player: PlayerId, source: ObjectId, destination: SearchDestination, find: Find, eligible: impl Fn(&dyn Query, ObjectId) -> bool) -> Result<Vec<ObjectId>, Suspend> {
    let found = search_library(g, player, player, source, destination, find.count, find.optional, eligible)?;
    if find.reveal {
        g.reveal(player, &found);
    }
    Ok(found)
}

/// "Search [a player's] library for [cards] and exile them, then that player shuffles" (CR 701.23, 701.23f, 406.2):
/// `searcher` searches the library of `owner` (the same player, or another), and the cards found, together with `also`
/// (cards the same search found in other zones, such as the graveyard and hand: CR 701.23a lets a search span zones),
/// go to exile at once, `also` first, as an exile (`ZoneReason::Exile`): the search only finds the cards, and "exile
/// them" is the exile. `also` exists so that the cards found in other zones leave in the same simultaneous move as the
/// library finds, not one move before or after. Each card goes to its owner's exile. The new objects, in that order, so
/// a caller can tell which of `also` were exiled. Deadly Cover-Up searches another player's
/// library and exiles what it finds with the graveyard and hand cards of the same search; Ugin, Eye of the Storms' -11
/// searches its controller's own.
pub fn search_to_exile(
    g: &mut dyn Game,
    searcher: PlayerId,
    owner: PlayerId,
    source: ObjectId,
    how: Find,
    also: &[ObjectId],
    eligible: impl Fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let found = search_library(g, searcher, owner, source, SearchDestination::Exile, how.count, how.optional, eligible)?;
    if how.reveal {
        g.reveal(searcher, &found);
    }
    let moves: Vec<(ObjectId, Zone)> = also.iter().chain(found.iter()).map(|&c| (c, Zone::exile(g.obj(c).owner))).collect();
    let moved = g.move_together(&moves, ZoneReason::Exile);
    g.shuffle(owner);
    Ok(moved)
}

/// "Search your library for [cards], put them into your hand, then shuffle" (CR 701.23): the cards found, as they are
/// in the hand. Demonic Tutor (`Find::ANY`) and Expedition Map (`Find::ONE.revealed()`) print it.
pub fn search_to_hand(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    how: Find,
    eligible: impl Fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let found = find(g, player, source, SearchDestination::Hand, how, eligible)?;
    let moves: Vec<(ObjectId, Zone)> = found.iter().map(|&c| (c, Zone::hand(player))).collect();
    let moved = g.move_together(&moves, ZoneReason::Search);
    g.shuffle(player);
    Ok(moved)
}

/// "Search your library for [cards], put them into your graveyard, then shuffle" (CR 701.23). Entomb and Goblin
/// Engineer print it.
pub fn search_to_graveyard(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    how: Find,
    eligible: impl Fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let found = find(g, player, source, SearchDestination::Graveyard, how, eligible)?;
    let moves: Vec<(ObjectId, Zone)> = found.iter().map(|&c| (c, Zone::graveyard(player))).collect();
    let moved = g.move_together(&moves, ZoneReason::Search);
    g.shuffle(player);
    Ok(moved)
}

/// "Search your library for [cards], put them onto the battlefield [tapped], then shuffle" (CR 701.23): the permanents
/// that arrived. The fetch lands (Flooded Strand, Arid Mesa), Tinker and Natural Order print it.
pub fn search_to_battlefield(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    how: Find,
    tapped: bool,
    eligible: impl Fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let found = find(g, player, source, if tapped { SearchDestination::BattlefieldTapped } else { SearchDestination::Battlefield }, how, eligible)?;
    let mut arrived = Vec::new();
    for card in found {
        arrived.push(g.put_onto_battlefield(card, player, tapped)?);
    }
    g.shuffle(player);
    Ok(arrived)
}

/// "Search your library for [a card], then shuffle and put that card on top" (CR 701.23): the library is shuffled
/// and the card placed on top; it never changes zones. Enlightened Tutor and Vampiric Tutor print it.
pub fn search_to_top(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    how: Find,
    eligible: impl Fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let found = find(g, player, source, SearchDestination::TopOfLibrary, how, eligible)?;
    g.shuffle(player);
    if !found.is_empty() {
        g.arrange_library(player, &found, &[]);
    }
    Ok(found)
}

/// What becomes of the cards looked at and not taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rest {
    /// On the bottom of the library in a random order (CR 401.4): nobody is asked for an order.
    BottomRandom,
    /// On the bottom in any order the player chooses, the first lowest.
    BottomChosen,
}

/// "Look at the top `count` cards of your library. Put between `min` and `max` of them that pass `wanted` into your hand
/// and the rest on the bottom of your library" (CR 401.2, 701.20-ish "look"): `reveal` makes it "you may reveal". A
/// library with fewer cards shows what it has (CR 609.3), and `min` and `max` shrink to the cards that qualify. With
/// nothing qualifying the player is still asked, to see the cards (the look is the instruction, the taking only its
/// option), and the hand takes none. `wanted` may capture the words of the source's text (CR 612). The cards taken, as they are in the hand. Once Upon a Time and Narset, Parter of
/// Veils (one, revealed, random), Dig Through Time and Stock Up (two, chosen order) print it.
pub fn look_at_top(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    count: usize,
    (min, max): (u8, u8),
    reveal: bool,
    wanted: impl Fn(&Chars) -> bool,
    rest: Rest,
) -> Result<Vec<ObjectId>, Suspend> {
    let looked = top_of_library(g, player, count);
    if looked.is_empty() {
        return Ok(Vec::new());
    }
    let takeable: Vec<ObjectId> = looked.iter().copied().filter(|&c| wanted(&g.chars(c))).collect();
    let min = min.min(takeable.len() as u8);
    let max = max.min(takeable.len() as u8);
    let ask = RequestKind::DistributeCards {
        source,
        cards: looked.clone(),
        destinations: vec![
            Destination { kind: DestinationKind::Hand, min, max, eligible: takeable },
            Destination { kind: DestinationKind::LibraryBottom, min: 0, max: u8::MAX, eligible: looked.clone() },
        ],
        may_shuffle: false,
    };
    let (taken, bottom) = match g.ask(player, ask)? {
        Answer::Placements { placements, .. } => {
            (placements.first().cloned().unwrap_or_default(), placements.get(1).cloned().unwrap_or_default())
        }
        _ => (Vec::new(), Vec::new()),
    };
    let taken = if taken.is_empty() {
        taken
    } else {
        if reveal {
            g.reveal(player, &taken);
        }
        g.return_to_hand(&taken)
    };
    match rest {
        Rest::BottomChosen => g.arrange_library(player, &[], &bottom),
        Rest::BottomRandom => rest_to_bottom_in_random_order(g, player, &looked),
    }
    Ok(taken)
}

/// "Put the rest on the bottom of your library in a random order" (CR 401.4) after looking at `looked`, cards of `player`'s
/// library: those still in it, a card taken having left. Narset, Parter of Veils, Once Upon a Time and United Battlefront print it.
pub fn rest_to_bottom_in_random_order(g: &mut dyn Game, player: PlayerId, looked: &[ObjectId]) {
    let rest: Vec<ObjectId> = looked.iter().copied().filter(|c| is_live(g, *c) && g.obj(*c).zone == Zone::library(player)).collect();
    g.bottom_in_random_order(player, &rest);
}

/// "Reveal the top `count` cards of your library. You may put a card that passes `wanted` from among them into your hand.
/// Put the rest into your graveyard" (CR 701.20, 400.3): the rest go in the order they were revealed. With nothing to take
/// nobody is asked. Satyr Wayfinder (a land card) and Malevolent Rumble (a permanent card) print it.
pub fn reveal_top_take_one_rest_to_graveyard(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    count: usize,
    wanted: fn(&Chars) -> bool,
) -> Result<(), Suspend> {
    let revealed = top_of_library(g, player, count);
    if revealed.is_empty() {
        return Ok(());
    }
    g.reveal(player, &revealed);
    let takeable: Vec<ObjectId> = revealed.iter().copied().filter(|&c| wanted(&g.chars(c))).collect();
    let mut taken = Vec::new();
    if !takeable.is_empty() {
        let ask = RequestKind::DistributeCards {
            source,
            cards: revealed.clone(),
            destinations: vec![
                Destination { kind: DestinationKind::Hand, min: 0, max: 1, eligible: takeable },
                Destination { kind: DestinationKind::Graveyard, min: 0, max: u8::MAX, eligible: revealed.clone() },
            ],
            may_shuffle: false,
        };
        if let Answer::Placements { placements, .. } = g.ask(player, ask)? {
            taken = placements[0].clone();
        }
    }
    g.return_to_hand(&taken);
    let rest: Vec<ObjectId> = revealed.into_iter().filter(|c| !taken.contains(c)).collect();
    g.put_into_graveyard(&rest);
    Ok(())
}

/// "Mill `count` cards. You may put a [`kind`] card from among them into your hand" (CR 701.17, 701.17c): the cards milled
/// are the ones it may take from, as they are where they went (a graveyard, or exile by a replacement effect); `wanted`
/// says which of them qualify, `kind` is how the question words it. Nobody is asked when none qualifies, and choosing none
/// declines. Six (a land), Tetzin, Gnome Champion (an artifact), Barrowgoyf (a creature, after its own
/// "you may mill") and Dredger's Insight (an artifact, creature, or land) print it.
pub fn mill_then_may_take_into_hand(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    count: u32,
    kind: &'static str,
    wanted: fn(&Chars) -> bool,
) -> Result<(), Suspend> {
    let milled = g.mill(player, count);
    let takeable: Vec<ObjectId> = milled.iter().copied().filter(|&c| wanted(&g.chars(c))).collect();
    let chosen = crate::choose::cards_from_kind(g, player, source, Purpose::PutIntoHand, kind, milled, takeable, 1, true)?;
    g.return_to_hand(&chosen);
    Ok(())
}

/// "In any order" (CR 401.4): `player` puts `cards` on one `end` of their library (`DestinationKind::LibraryTop` or
/// `LibraryBottom`) in the order they choose, the first lowest. Nobody is asked about fewer than two cards. The cards in
/// that order, for `put_into_library` or `arrange_library`. Bow of Nylea and Valakut Awakening put cards on the bottom,
/// Lim-Dul's Vault on either end.
pub fn ask_library_order(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    cards: &[ObjectId],
    end: DestinationKind,
) -> Result<Vec<ObjectId>, Suspend> {
    if cards.len() < 2 {
        return Ok(cards.to_vec());
    }
    let ask = RequestKind::DistributeCards {
        source,
        cards: cards.to_vec(),
        destinations: vec![Destination { kind: end, min: 0, max: u8::MAX, eligible: cards.to_vec() }],
        may_shuffle: false,
    };
    Ok(match g.ask(player, ask)? {
        Answer::Placements { placements, .. } if placements.first().is_some_and(|p| p.len() == cards.len()) => placements[0].clone(),
        _ => cards.to_vec(),
    })
}

/// "Reveal the top card of your library and put it into your hand" (CR 701.20, 400.3): a move, not a draw. The card, as
/// it is in the hand, and its mana value as it was revealed (CR 608.2h); nothing from an empty library. Dark Confidant,
/// Ad Nauseam and Caustic Bronco make the player lose life equal to that value.
pub fn reveal_top_to_hand(g: &mut dyn Game, player: PlayerId) -> Option<(ObjectId, u32)> {
    let &top = top_of_library(g, player, 1).first()?;
    let value = g.mana_value(top);
    g.reveal(player, &[top]);
    let moved = g.return_to_hand(&[top]);
    Some((moved.first().copied().unwrap_or(top), value))
}

/// Whether `card` is still the card it was and in exile (CR 400.7): "if it's still exiled", after it may have been cast or
/// played from there, or moved by a replacement effect. Beseech the Mirror, Etali, Primal Conqueror, Tersa Lightshatter and
/// Dragonhawk, Fate's Tempest ask it.
pub fn is_still_exiled(q: &dyn Query, card: ObjectId) -> bool {
    is_live(q, card) && q.obj(card).zone.is(ZoneKind::Exile)
}

/// The `cards` that are still exiled (`is_still_exiled`), in order. Crabomination, Dragonhawk, Fate's Tempest and cascade's
/// "put the rest on the bottom" ask it.
pub fn still_exiled(q: &dyn Query, cards: &[ObjectId]) -> Vec<ObjectId> {
    cards.iter().copied().filter(|&c| is_still_exiled(q, c)).collect()
}

/// "Exile the top `count` cards of `from`'s library. `who` may play them" under `grant` (CR 406, 305.1, 601.3): the cards
/// leave together, face up, and each that really reached exile (a replacement may send it elsewhere) can be played
/// under the grant, a land among them under the ordinary land-play rules. The cards still in exile. Laelia, the Blade
/// Reforged (one, this turn), Showdown of the Skalds (four, until the end of your next turn) and Mind's Desire print it.
pub fn exile_top_may_play(g: &mut dyn Game, from: PlayerId, count: usize, who: PlayerId, grant: PlayGrant) -> Vec<ObjectId> {
    let top = top_of_library(g, from, count);
    let mut exiled = Vec::new();
    for card in g.exile_all(&top) {
        if g.obj(card).zone.is(ZoneKind::Exile) {
            g.allow_play(card, who, grant);
            exiled.push(card);
        }
    }
    exiled
}

/// "Exile the top `count` cards of your library. Until the end of your next turn, you may play [it / those cards]"
/// (`exile_top_may_play`): the grant is to play, not only cast, a land among them under the ordinary land-play rules, until the
/// cleanup step of `player`'s next turn (CR 514.2). The cards still in exile. Cori Mountain Monastery (one), Clockwork
/// Percussionist (one), The Legend of Roku (three) and Showdown of the Skalds (four) print it.
pub fn exile_top_play_until_end_of_next_turn(g: &mut dyn Game, player: PlayerId, count: usize) -> Vec<ObjectId> {
    let grant = PlayGrant { instead: None, lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST };
    exile_top_may_play(g, player, count, player, grant)
}

/// "[One] of `items` chosen at random" (CR 701.9c, 705): one is drawn from the game's own generator, so the same game picks
/// the same one; no draw is made when there is no choice. None from an empty list. Crabomination picks a card at random
/// from a graveyard and a hand, Chaos Defiler one of the creatures chosen, Ursine Monstrosity an opponent and Tersa
/// Lightshatter a card of her graveyard.
pub fn random_of<T: Copy>(g: &mut dyn Game, items: &[T]) -> Option<T> {
    match items.len() {
        0 => None,
        1 => items.first().copied(),
        n => items.get(g.choose_at_random(n)).copied(),
    }
}

/// Whether `card` has the name `wanted`, the name of a card (CR 201.2a). A split card has the name of each half in
/// common with a card of that name (CR 201.2a, 201.4b: a player naming a split card names one of its halves).
pub fn card_has_name(q: &dyn Query, card: CardId, wanted: CardId) -> bool {
    let name = q.card(wanted).name;
    let def = q.card(card);
    def.name == name || def.split.is_some_and(|(left, right)| left.name == name || right.name == name)
}

/// The names the object `id` has (CR 201.2a): a Room on the battlefield those of its unlocked doors alone, and none at all
/// with neither unlocked (CR 709.5); any other split card both halves' (CR 709.4a); a copy that says "except its name is X"
/// only that (CR 707.9b); and anything else its own name, one for each of the two faces' names of a card written "A // B".
pub fn object_names(q: &dyn Query, id: ObjectId) -> Vec<&'static str> {
    let c = q.chars(id);
    let obj = q.obj(id);
    if obj.copy.is_some_and(|copy| copy.name.is_some()) {
        return vec![c.name];
    }
    let def = q.card(c.card);
    if def.is_room() && obj.zone.is(ZoneKind::Battlefield) {
        return def.unlocked_doors(obj.marks.doors).map(|half| half.name).collect();
    }
    if let Some((left, right)) = def.split {
        return vec![left.name, right.name];
    }
    c.name.split(" // ").filter(|name| !name.is_empty()).collect()
}

/// Whether the object `id` has the name `wanted` (CR 201.2a, 709.4a): "an object has the chosen name if one of its names is
/// the chosen name" (`object_names`). Anointed Peacekeeper's "sources with the chosen name", Tamiyo, Collector of Tales and any
/// card that asks of a permanent, a spell or a card in a zone whether it is named so.
pub fn object_has_name(q: &dyn Query, id: ObjectId, wanted: CardId) -> bool {
    let name = q.card(wanted).name;
    object_names(q, id).contains(&name)
}

/// "Exile cards from the top of your library until you exile a nonland card [that `qualifies`]" (CR 701.57a, 702.85a): each
/// card is exiled one at a time, face up, from the top until one is a nonland card `qualifies` accepts, or the library
/// is empty. Every card that really reached exile, in order (the qualifying one last), and the qualifying card if there
/// was one; a card a replacement effect sent elsewhere is neither. Cascade (a lesser mana value than the spell's) and
/// discover (a mana value of N or less) use it.
pub fn exile_until_nonland(
    g: &mut dyn Game,
    player: PlayerId,
    qualifies: &dyn Fn(&dyn Query, ObjectId) -> bool,
) -> (Vec<ObjectId>, Option<ObjectId>) {
    let mut exiled = Vec::new();
    while let Some(&top) = g.state().zone(Zone::library(player)).first() {
        let moved = g.exile(top);
        if !g.obj(moved).zone.is(ZoneKind::Exile) {
            continue;
        }
        exiled.push(moved);
        if !g.chars(moved).is(CardType::Land) && qualifies(&*g, moved) {
            return (exiled, Some(moved));
        }
    }
    (exiled, None)
}
