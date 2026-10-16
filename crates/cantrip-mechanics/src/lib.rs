//! Shared mechanics (docs/engine-design.md, "Layers"): code that is the same across cards and differs only in
//! their data. Built from `cantrip-api` alone. Admission is CLAUDE.md's test: a second printed card would use it
//! exactly as written.

pub mod adapt;
pub mod affinity;
pub mod amass;
pub mod attach;
pub mod bargain;
pub mod behold;
pub mod blight;
pub mod bestow;
pub mod boast;
pub mod battle_cry;
pub mod backup;
pub mod dash;
pub mod combat;
pub mod copy;
pub mod channel;
pub mod choose;
pub mod cipher;
pub mod chosen_type;
pub mod hand;
pub mod history;
pub mod library;
pub mod impending;
pub mod firebending;
pub mod flashback;
pub mod flurry;
pub mod goyf;
pub mod harness;
pub mod hideaway_play;
pub mod cleave;
pub mod crew;
pub mod devoid;
pub mod double;
pub mod earthbend;
pub mod escalate;
pub mod fading;
pub mod escape;
pub mod eternalize;
pub mod evoke;
pub mod for_mirrodin;
pub mod delayed;
pub mod effects;
pub mod foretell;
pub mod kicker;
pub mod level_up;
pub mod lands;
pub mod mana;
pub mod manifest_dread;
pub mod mayhem;
pub mod mobilize;
pub mod ninjutsu;
pub mod offspring;
pub mod persist;
pub mod overload;
pub mod paradigm;
pub mod plot;
pub mod ravenous;
pub mod reconfigure;
pub mod regenerate;
pub mod saddle;
pub mod replicate;
pub mod retrace;
pub mod ripple;
pub mod replacements;
pub mod statics;
pub mod storm;
pub mod spree;
pub mod sneak;
pub mod spectacle;
pub mod suspend;
pub mod typecycling;
pub mod unearth;
pub mod triggers;
pub mod warp;
pub mod targets;
pub mod text_change;
pub mod tokens;
pub mod untap;
pub mod vote;

use cantrip_api::*;

/// A player chooses `count` cards of their hand (all of it if they have fewer) and discards them together
/// (CR 701.9). Asked even when the choice is forced, except of an empty hand.
pub fn discard_chosen(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: u8) -> Result<Vec<ObjectId>, Suspend> {
    let chosen = choose_from_hand(g, player, source, count)?;
    g.discard_together(&chosen);
    Ok(chosen)
}

/// The cards a player chooses from their own hand for a discard, without discarding them yet.
pub fn choose_from_hand(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: u8) -> Result<Vec<ObjectId>, Suspend> {
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    choose::cards(g, player, source, Purpose::Discard, hand, count, false)
}

/// "[Player] discards `count` cards at random" (CR 701.9b; Hymn to Tourach, Mind Twist): the hand put in a random order
/// (Fisher-Yates, as the C# engine's `Randomize`) and the first `count` taken; a smaller hand is discarded whole. They
/// leave together, as one discard (CR 701.9a).
pub fn discard_at_random(g: &mut dyn Game, player: PlayerId, count: usize) {
    let mut hand = g.state().zone(Zone::hand(player)).to_vec();
    let count = hand.len().min(count);
    if count == 0 {
        return;
    }
    for i in (1..hand.len()).rev() {
        let j = g.random(i as u32 + 1) as usize;
        hand.swap(i, j);
    }
    g.discard_together(&hand[..count]);
}

/// "[Player] sacrifices a [kind]" (CR 701.21): chosen by that player when there is a choice, taken when there is not.
pub fn sacrifice_chosen(
    g: &mut dyn Game,
    player: PlayerId,
    source: ObjectId,
    count: u8,
    kind: fn(&Chars) -> bool,
) -> Result<(), Suspend> {
    let eligible: Vec<ObjectId> = g.permanents_of(player).into_iter().filter(|&id| kind(&g.chars(id))).collect();
    sacrifice_one_of(g, player, source, count, eligible)
}

/// "Each opponent sacrifices `count` [kind] of their choice" (CR 701.21a, 101.4): in turn order from the active player,
/// each opponent chooses when there is a choice and sacrifices what they chose before the next is asked. `kind` is judged
/// on the object as well as its characteristics, so it can ask whether a permanent is a token. Portal to Phyrexia (three
/// creatures) and Sheoldred's Edict (a nontoken creature, a creature token, a planeswalker) print it.
pub fn each_opponent_sacrifices(
    g: &mut dyn Game,
    controller: PlayerId,
    source: ObjectId,
    count: u8,
    kind: fn(&dyn Query, ObjectId) -> bool,
) -> Result<(), Suspend> {
    for who in g.state().apnap().filter(|&p| p != controller).collect::<Vec<_>>() {
        let eligible: Vec<ObjectId> = g.permanents_of(who).into_iter().filter(|&id| kind(&*g, id)).collect();
        sacrifice_one_of(g, who, source, count, eligible)?;
    }
    Ok(())
}

fn sacrifice_one_of(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: u8, eligible: Vec<ObjectId>) -> Result<(), Suspend> {
    if eligible.len() <= count as usize {
        g.sacrifice(&eligible);
        return Ok(());
    }
    let chosen = choose::cards(g, player, source, Purpose::Sacrifice, eligible, count, false)?;
    g.sacrifice(&chosen);
    Ok(())
}

fn untapped_creature_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.controller(id) == asker.controller && !q.obj(id).tapped && q.chars(id).is(CardType::Creature)
}

/// Delirium (CR 702.138a): four or more card types among the cards in `player`'s graveyard. Fear of Missing Out and
/// Unholy Heat read it.
pub fn delirium(q: &dyn Query, player: PlayerId) -> bool {
    let mut types = TypeSet::NONE;
    for &id in q.state().zone(Zone::graveyard(player)) {
        for t in q.chars(id).types.iter() {
            types = types.with(t);
        }
    }
    types.iter().count() >= 4
}

/// "Tap an untapped creature you control" as a cost (CR 118.3): one creature its payer chooses, summoning sick or not,
/// since it is not the {T} symbol of that creature (CR 107.5, 302.6). Earthcraft, Kyren Negotiations and Springleaf Drum
/// print it.
pub const TAP_AN_UNTAPPED_CREATURE: Cost = Cost::tap_untapped(1, untapped_creature_you_control);

/// "Target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its
/// mana cost" (CR 702.34a): its controller may cast it from the graveyard this turn for its mana cost, and it is exiled
/// instead of going anywhere else as it leaves the stack. Snapcaster Mage and Flashback print it.
pub fn grant_flashback(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Object(card) = item.targets[0]
    {
        g.allow_play(card, item.controller, PlayGrant { expiry: Expiry::EndOfTurn, exile_after: true, ..PlayGrant::CAST });
    }
    Ok(())
}

/// Connive (CR 701.50): draw a card, then discard a card; if a nonland card was discarded, put a +1/+1 counter on
/// the creature. A creature that has left the battlefield still connives, and takes no counter; its controller is
/// the one it had last (CR 608.2h).
pub fn connive(g: &mut dyn Game, creature: ObjectId, source: ObjectId) -> Result<(), Suspend> {
    connive_n(g, creature, source, 1)
}

/// Connive N (CR 701.50): draw N cards, then discard N (as many as the hand holds), together, and a +1/+1 counter
/// goes on the creature for each nonland card discarded. Raffine, Scheming Seer connives X; Ledger Shredder, 1.
pub fn connive_n(g: &mut dyn Game, creature: ObjectId, source: ObjectId, count: u8) -> Result<(), Suspend> {
    let player = g.obj(creature).controller;
    g.draw(player, count as u32);
    let chosen = choose_from_hand(g, player, source, count)?;
    let nonland = chosen.iter().filter(|&&c| !g.chars(c).is(CardType::Land)).count() as i16;
    g.discard_together(&chosen);
    if nonland > 0 && g.is_on_battlefield(creature) && g.obj(creature).became.is_none() {
        g.add_counters(creature, CounterKind::PlusOne, nonland);
    }
    Ok(())
}

/// "Exile [it], then return it to the battlefield transformed under [a player's] control" (CR 712.14a): Ajani,
/// Nacatl Pariah, Fable of the Mirror-Breaker and The Legend of Roku print it. Nothing if the permanent has left the
/// battlefield already, since the new object is not the one the ability means (CR 400.7).
pub fn exile_and_return_transformed(g: &mut dyn Game, permanent: ObjectId, under: PlayerId) -> Result<(), Suspend> {
    if g.obj(permanent).became.is_some() || !g.is_on_battlefield(permanent) {
        return Ok(());
    }
    // A token that is exiled is gone for good (CR 111.7): only a card comes back.
    let token = g.obj(permanent).token;
    let exiled = g.exile(permanent);
    if !token && g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.put_onto_battlefield_transformed(exiled, under)?;
    }
    Ok(())
}

/// Exiles the targeted card, if it is still a legal target.
pub fn exile_target(g: &mut dyn Game, item: &StackItem, slot: usize) -> bool {
    if !g.target_legal(item, slot) {
        return false;
    }
    if let Target::Object(id) = item.targets[slot] {
        g.exile(id);
        return true;
    }
    false
}

/// Destroys the targeted permanent, if it is still a legal target (CR 701.8, 608.2b); the permanent, so a caller can go on
/// to read what it was (its controller, its power). Disenchant and Vindicate print it.
pub fn destroy_target(g: &mut dyn Game, item: &StackItem, slot: usize) -> Option<ObjectId> {
    let id = targets::object(g, item, slot)?;
    g.destroy(id);
    Some(id)
}

/// "Destroy target [thing]. It can't be regenerated" (CR 701.8, 701.19c): `destroy_target` that a regeneration shield does
/// not save. Snuff Out and Echoing Ruin print it.
pub fn destroy_target_no_regen(g: &mut dyn Game, item: &StackItem, slot: usize) -> Option<ObjectId> {
    let id = targets::object(g, item, slot)?;
    g.destroy_no_regen(id);
    Some(id)
}

/// Counters the targeted spell, if it is still a legal target (CR 701.6, 608.2b): Counterspell and Annul print it.
pub fn counter_target(g: &mut dyn Game, item: &StackItem, slot: usize) -> Option<ObjectId> {
    let spell = targets::object(g, item, slot)?;
    g.counter(spell);
    Some(spell)
}

/// "Counter target spell unless its controller pays `cost`" (CR 118.12, 701.6): the spell's controller is asked, and may
/// make mana for it while asked; the cost is judged as the effect resolves, so a card whose cost depends on the board
/// (Override's {1} per artifact) reads it first and passes it. Mana Leak and Miscalculation print it with a fixed cost.
pub fn counter_unless_pays(g: &mut dyn Game, item: &StackItem, slot: usize, cost: ManaCost) -> Result<Option<ObjectId>, Suspend> {
    let Some(spell) = targets::object(g, item, slot) else { return Ok(None) };
    let payer = g.controller(spell);
    if !g.unless_pays(payer, item.object, spell, cost)? {
        g.counter(spell);
    }
    Ok(Some(spell))
}

/// Exiles the targeted permanent until `source` leaves the battlefield (CR 610.3, 400.7): a returning card is a new
/// object. The permanent that was exiled, if the target was still legal. Aang's Iceberg and Conclave Tribunal print it
/// (Portable Hole and Sheltered by Ghosts as well).
pub fn exile_target_until_leaves(g: &mut dyn Game, item: &StackItem, slot: usize) -> Option<ObjectId> {
    let id = targets::object(g, item, slot)?;
    g.exile_with(&[id], item.object, true);
    Some(id)
}

/// Airbend (CR 701.65a): exiles the resolving ability's legal targets together, and each card exiled this way may be
/// cast by its owner for {2} rather than its mana cost for as long as it stays exiled. A token or a copy is exiled and
/// gone: only a card can be cast again. Appa, Steadfast Guardian and Aang, Swift Savior print it.
pub fn airbend(g: &mut dyn Game, item: &StackItem) {
    let targets: Vec<ObjectId> =
        (0..item.target_count as usize).filter(|&slot| g.target_legal(item, slot)).filter_map(|slot| item.targets[slot].object()).collect();
    for exiled in g.exile_all(&targets) {
        let obj = *g.obj(exiled);
        if obj.zone.is(ZoneKind::Exile) && !obj.token && !obj.card_copy {
            g.allow_play(exiled, obj.owner, PlayGrant { instead: Some(mana("{2}")), ..PlayGrant::CAST });
        }
    }
}

/// Deals damage to the target in `slot`, if it is still legal.
pub fn damage_target(g: &mut dyn Game, item: &StackItem, slot: usize, amount: u16) {
    if g.target_legal(item, slot) {
        g.damage(item.object, item.targets[slot], amount);
    }
}

/// Deals each target of the resolving item its own share of the damage it was divided into (CR 601.2d, 120.3): "N damage
/// divided as you choose among [targets]". The shares were fixed as the spell or ability was put on the stack, a target
/// that has since become illegal is left out with its share (CR 608.2b), and the rest is dealt together as one event.
/// Pyrokinesis, Fire Covenant, Fury and Shatterskull Smashing print it.
pub fn damage_divided(g: &mut dyn Game, item: &StackItem) {
    let hits: Vec<(ObjectId, Target, u16)> = (0..item.target_count as usize)
        .filter(|&slot| g.target_legal(item, slot) && item.share(slot) > 0)
        .map(|slot| (item.object, item.targets[slot], item.share(slot)))
        .collect();
    g.damage_together(&hits);
}

/// A one-color choice (CR 105.1), asked of `player`.
pub fn choose_color(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<Color, Suspend> {
    let answer = g.ask(player, RequestKind::ChooseColors { source, choices: ColorSet(0b11111), count: 1 })?;
    Ok(match answer {
        Answer::Colors(colors) => colors.first().copied().unwrap_or(Color::White),
        _ => Color::White,
    })
}

/// `searcher` searches the library of `owner` (CR 701.23, 701.23f; the two are the same player for "search your
/// library") for up to `count` cards (exactly as many as can be found, when not `optional`) that pass `eligible`.
/// Asked only when the library has cards; the search is recorded either way. The library is listed bottom first, as
/// the C# engine lists it. `destination` is where the cards found are going, for the question to say.
pub fn search_library(
    g: &mut dyn Game,
    searcher: PlayerId,
    owner: PlayerId,
    source: ObjectId,
    destination: SearchDestination,
    count: u8,
    optional: bool,
    eligible: impl Fn(&dyn Query, ObjectId) -> bool,
) -> Result<Vec<ObjectId>, Suspend> {
    let mut library: Vec<ObjectId> = g.state().zone(Zone::library(owner)).to_vec();
    library.reverse();
    let found = if library.is_empty() {
        Vec::new()
    } else {
        let can: Vec<ObjectId> = library.iter().copied().filter(|&c| eligible(g, c)).collect();
        let count = if optional { count.max(1) } else { count.max(1).min(can.len() as u8) };
        match g.ask(searcher, RequestKind::SearchLibrary { source, library, eligible: can, count, optional, destination })? {
            Answer::Cards(cards) => cards,
            _ => Vec::new(),
        }
    };
    g.searched(searcher, owner);
    Ok(found)
}

/// The top `count` cards of a player's library, top first.
pub fn top_of_library(g: &dyn Query, player: PlayerId, count: usize) -> Vec<ObjectId> {
    g.state().zone(Zone::library(player)).iter().take(count).copied().collect()
}

/// "[It] gets +P/+T until end of turn" (CR 611.2): a continuous effect from the resolving ability's source.
pub fn pump(g: &mut dyn Game, source: ObjectId, creature: ObjectId, power: i16, toughness: i16) {
    if g.is_on_battlefield(creature) && g.obj(creature).became.is_none() {
        g.add_effect(source, creature, EffectKind::Pump(power, toughness), Duration::EndOfTurn);
    }
}

/// Scry N (CR 701.22): look at the top N cards of your library, put any number on the bottom in any order and the
/// rest back on top in any order. Nothing is asked of an empty library. Undercity's Lost Well prints "scry 2".
pub fn scry(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: usize) -> Result<(), Suspend> {
    let looked = top_of_library(g, player, count);
    if looked.is_empty() {
        return Ok(());
    }
    let ask = RequestKind::DistributeCards {
        source,
        cards: looked.clone(),
        destinations: vec![
            Destination { kind: DestinationKind::LibraryTop, min: 0, max: u8::MAX, eligible: looked.clone() },
            Destination { kind: DestinationKind::LibraryBottom, min: 0, max: u8::MAX, eligible: looked },
        ],
        may_shuffle: false,
    };
    if let Answer::Placements { placements, .. } = g.ask(player, ask)? {
        g.arrange_library(player, &placements[0], &placements[1]);
    }
    Ok(())
}

/// Surveil N (CR 701.25): look at the top N cards of your library, put any number into your graveyard (together, in
/// the order chosen, so the last is on top) and the rest back on top in any order. Nothing is asked of an empty
/// library. Second card: Consider, Thought Monitor's surveil, any "surveil 2".
pub fn surveil(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: usize) -> Result<(), Suspend> {
    let looked = top_of_library(g, player, count);
    if looked.is_empty() {
        return Ok(());
    }
    let ask = RequestKind::DistributeCards {
        source,
        cards: looked.clone(),
        destinations: vec![
            Destination { kind: DestinationKind::LibraryTop, min: 0, max: u8::MAX, eligible: looked.clone() },
            Destination { kind: DestinationKind::Graveyard, min: 0, max: u8::MAX, eligible: looked },
        ],
        may_shuffle: false,
    };
    if let Answer::Placements { placements, .. } = g.ask(player, ask)? {
        let moves: Vec<(ObjectId, Zone)> = placements[1].iter().map(|&c| (c, Zone::graveyard(player))).collect();
        g.move_together(&moves, ZoneReason::Other);
        g.arrange_library(player, &placements[0], &[]);
    }
    Ok(())
}

/// What "becomes a P/T [colors] [subtype] creature with [keywords]" makes of a permanent (CR 205.1b, 613.1). `colors`
/// and `subtype` are `None` when the text names none ("becomes a 4/4 artifact creature"): the permanent keeps its own.
pub struct Animation {
    pub power: i16,
    pub toughness: i16,
    pub colors: Option<ColorSet>,
    pub subtype: Option<Subtype>,
    pub keywords: Keywords,
    /// A triggered or activated ability the permanent is given for the duration (CR 613.1f), if the text gives one.
    pub ability: Option<AbilityRef>,
}

/// "[Until end of turn,] this land becomes a P/T [colors] [subtype] creature with [keywords]. It's still a land", for the
/// `duration` the text gives (Great Hall of the Biblioplex has none: `Duration::Indefinite`): the creature type is added
/// (layer 4) with its subtype, the colors replace its own (5), the keywords and the ability are granted (6) and the base
/// power and toughness set (7b), so counters and pumps count on top. Celestial Colonnade and Creeping Tar Pit print it
/// until end of turn, Firdoch Core names no colors or creature type and leaves those two out, and Great Hall of the
/// Biblioplex gives a triggered ability.
pub fn animate(g: &mut dyn Game, source: ObjectId, permanent: ObjectId, a: &Animation, duration: Duration) {
    if !g.is_on_battlefield(permanent) || g.obj(permanent).became.is_some() {
        return;
    }
    g.add_effect(source, permanent, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), duration);
    if let Some(subtype) = a.subtype {
        g.add_effect(source, permanent, EffectKind::AddSubtype(subtype), duration);
    }
    if let Some(colors) = a.colors {
        g.add_effect(source, permanent, EffectKind::SetColors(colors), duration);
    }
    g.add_effect(source, permanent, EffectKind::AddKeywords(a.keywords), duration);
    if let Some(ability) = a.ability {
        g.add_effect(source, permanent, EffectKind::AddAbility(ability), duration);
    }
    g.add_effect(source, permanent, EffectKind::SetPt(a.power, a.toughness), duration);
}

/// Whether two objects share a name (CR 201.2): an object with two names (a split or double-faced card, "A // B") shares
/// one with anything that has either, a Room on the battlefield has those of its unlocked doors alone (CR 709.5), and a
/// copy has the name it copies (`library::object_names`). Echoing Return, Echoing Ruin and Echoing Truth print "all other
/// [things] with the same name as that [thing]", and Bloodbond March and Deadly Cover-Up "with the same name".
pub fn same_name(q: &dyn Query, a: ObjectId, b: ObjectId) -> bool {
    let theirs = library::object_names(q, b);
    library::object_names(q, a).iter().any(|name| theirs.contains(name))
}

/// Explore (CR 701.44): the permanent's controller reveals the top card of their library; a land goes into their hand,
/// otherwise the permanent gets a +1/+1 counter (if it is still the permanent it was) and they may put the card into
/// their graveyard. Fanatical Offering's Map token and Seasoned Dungeoneer explore it.
pub fn explore(g: &mut dyn Game, permanent: ObjectId) -> Result<(), Suspend> {
    let who = g.obj(permanent).controller;
    let Some(&top) = top_of_library(g, who, 1).first() else { return Ok(()) };
    g.reveal(who, &[top]);
    if g.chars(top).is(CardType::Land) {
        g.move_to(top, Zone::hand(who));
        return Ok(());
    }
    if is_live(g, permanent) && g.is_on_battlefield(permanent) {
        g.add_counters(permanent, CounterKind::PlusOne, 1);
    }
    if matches!(g.ask(who, RequestKind::May { source: top, purpose: Purpose::PutIntoGraveyard })?, Answer::Yes) {
        g.move_to(top, Zone::graveyard(who));
    }
    Ok(())
}

/// "Look at the top `count` cards of your library. You may reveal a noncreature, nonland card from among them and put
/// it into your hand. Put the rest on the bottom of your library in a random order." Narset, Parter of Veils' -2 and
/// Thundertrap Trainer's enters ability. With no such card among them nobody is asked.
pub fn look_for_noncreature_nonland(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: usize) -> Result<(), Suspend> {
    look_for_one(g, player, source, count, |chars| !chars.is(CardType::Creature) && !chars.is(CardType::Land))
}

/// "Look at the top `count` cards of your library. You may reveal a creature or land card from among them and put it
/// into your hand. Put the rest on the bottom of your library in a random order." Once Upon a Time. With no such card
/// among them nobody is asked.
pub fn look_for_creature_or_land(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: usize) -> Result<(), Suspend> {
    look_for_one(g, player, source, count, |chars| chars.is(CardType::Creature) || chars.is(CardType::Land))
}

/// The look both of the above make: one card `wanted` may go to the hand, the rest to the bottom at random.
fn look_for_one(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: usize, wanted: fn(&Chars) -> bool) -> Result<(), Suspend> {
    library::look_at_top(g, player, source, count, (0, 1), true, wanted, library::Rest::BottomRandom)?;
    Ok(())
}

/// "Each player shuffles their hand and graveyard into their library, then draws seven cards": every hand and
/// graveyard moves together (CR 701.24), the libraries are shuffled, and each player draws in turn order from the
/// active player (CR 101.4); nobody is asked. Echo of Eons, Timetwister and Time Spiral print it.
pub fn shuffle_hands_and_graveyards_in_and_draw_seven(g: &mut dyn Game) {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let mut cards: Vec<ObjectId> = Vec::new();
    for &p in &players {
        cards.extend(g.state().zone(Zone::hand(p)).iter().chain(g.state().zone(Zone::graveyard(p)).iter()).copied());
    }
    g.put_into_library(&cards, LibraryPlace::Top);
    for &p in &players {
        g.shuffle(p);
    }
    for &p in &players {
        g.draw(p, 7);
    }
}

/// Hideaway N (CR 702.75a): look at the top N cards of the library, exile one of them (exactly one, with fewer than N
/// cards it is one of those there are), and put the rest on the bottom in a random order. The exiled card is linked to
/// `source` (CR 607.2a), so its own "play the exiled card" ability finds it with `Query::exiled_with`. Windbrisk Heights,
/// Shelldock Isle and Collector's Cage print it.
pub fn hideaway(g: &mut dyn Game, player: PlayerId, source: ObjectId, count: usize) -> Result<(), Suspend> {
    let looked = top_of_library(g, player, count);
    if looked.is_empty() {
        return Ok(());
    }
    let ask = RequestKind::DistributeCards {
        source,
        cards: looked.clone(),
        destinations: vec![
            Destination { kind: DestinationKind::Exile, min: 1, max: 1, eligible: looked.clone() },
            Destination { kind: DestinationKind::LibraryBottom, min: 0, max: u8::MAX, eligible: looked.clone() },
        ],
        may_shuffle: false,
    };
    let mut chosen = None;
    if let Answer::Placements { placements, .. } = g.ask(player, ask)? {
        chosen = placements.first().and_then(|p| p.first().copied());
    }
    // Everything not exiled goes to the bottom in a random order, whether or not the exile itself succeeds.
    let rest: Vec<ObjectId> = looked.into_iter().filter(|&c| Some(c) != chosen).collect();
    if let Some(card) = chosen {
        // Exiled face down (CR 406.3); the player who looked at it may go on looking.
        let exiled = g.exile_face_down(&[card], Some(source));
        g.look_at(player, &exiled);
    }
    g.bottom_in_random_order(player, &rest);
    Ok(())
}
