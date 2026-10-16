//! One-line effects as functions a `TriggeredDef` or `ActivatedDef` can name, parameterised the way `ward::<N>` is, so
//! a card does not write a wrapper around a single `Game` call. Each acts for the ability's controller or its source.

use cantrip_api::*;

/// "This creature enters with X +1/+1 counters on it" (CR 614.1c), for the X it was cast with; put onto the battlefield
/// without being cast, X is 0. Walking Ballista, Guiding Hydra and the ravenous creatures print it.
pub fn enters_with_x_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += entering.x as i16;
    Ok(())
}

/// "Creatures you control get +`power`/+`toughness` until end of turn" (CR 611.2, 611.2c): the creatures `item`'s controller
/// controls as it resolves, and no creature that enters later. The creatures, for what the rest of the sentence does to them
/// ("and gain haste", "untap those creatures"). Assault Formation, Eagles of the North, Fatehold Charm, Haze of Rage, Jeskai
/// Ascendancy, Lorehold Charm, Silk Web Weaver and Warhost's Frenzy print it.
pub fn pump_creatures_you_control(g: &mut dyn Game, item: &StackItem, power: i16, toughness: i16) -> Vec<ObjectId> {
    let creatures: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for &id in &creatures {
        crate::pump(g, item.object, id, power, toughness);
    }
    creatures
}

/// "Put `n` `kind` counters on each creature you control": on the creatures `player` controls as it is read, one after the
/// other. The creatures, for what the rest of the sentence does to "those creatures". Abandoned Air Temple, Elspeth,
/// Storm Slayer, Anduril, Narsil Reforged, Gavony Township, Germination Practicum, Ouroboroid and The Theorist, Jace
/// Beleren print it.
pub fn counters_on_creatures_you_control(g: &mut dyn Game, player: PlayerId, kind: CounterKind, n: i16) -> Vec<ObjectId> {
    let creatures: Vec<ObjectId> = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for &id in &creatures {
        g.add_counters(id, kind, n);
    }
    creatures
}

fn creatures_on_the_battlefield(g: &dyn Game) -> Vec<ObjectId> {
    g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect()
}

/// "Destroy all creatures." Every creature goes at once, in battlefield order (CR 701.8a, 101.4); regeneration and
/// indestructible are the verb's business. Day of Judgment prints it.
pub fn destroy_all_creatures(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let creatures = creatures_on_the_battlefield(g);
    g.destroy_all(&creatures);
    Ok(())
}

/// "Destroy all creatures. They can't be regenerated." (CR 701.19c). Wrath of God and Damnation print it.
pub fn destroy_all_creatures_no_regen(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let creatures = creatures_on_the_battlefield(g);
    g.destroy_all_no_regen(&creatures);
    Ok(())
}

/// "Draw N cards" (CR 121.1). Chromatic Star and Mind Stone print "draw a card"; Coveted Jewel and Bankrupt in Blood draw
/// three.
pub fn draw(g: &mut dyn Game, item: &StackItem, n: u32) -> Result<(), Suspend> {
    g.draw(item.controller, n);
    Ok(())
}

/// "Draw a card, then discard a card" (CR 121.1, 701.9): the controller draws, then chooses a card of their hand and
/// discards it. Currency Converter and Sphinx of False Conclusions print it.
pub fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    crate::discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

/// "You may discard a card. If you do, draw `n` cards" (CR 701.9, 121.1): the controller chooses a card of their hand to
/// discard or none; the draw follows a card that was actually discarded, and nothing is asked of an empty hand. Melded
/// Moxite and Bitter Reunion draw two, Giott, King of the Dwarves and Lorehold, the Historian draw one.
pub fn may_discard_to_draw(g: &mut dyn Game, item: &StackItem, n: u32) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    if let Some(&card) = crate::choose::cards(g, item.controller, item.object, Purpose::Discard, hand, 1, true)?.first() {
        g.discard(card);
        g.draw(item.controller, n);
    }
    Ok(())
}

/// "Discard a card, then draw N cards" (CR 701.9, 121.1): the controller chooses a card of their hand and discards it (nothing is
/// asked of an empty hand), then draws, whether or not they discarded. Fear of Missing Out (one card) and Case of the Crimson
/// Pulse (two) print it.
pub fn discard_then_draw(g: &mut dyn Game, item: &StackItem, n: u32) -> Result<(), Suspend> {
    crate::discard_chosen(g, item.controller, item.object, 1)?;
    g.draw(item.controller, n);
    Ok(())
}

/// "Discard your hand, then draw N cards" (CR 701.9, 121.1): the whole hand goes at once with no choice, then the draw
/// happens whatever the hand was. Anjes Ravager (three cards) and Case of the Crimson Pulse (two) print it.
pub fn discard_hand_then_draw(g: &mut dyn Game, item: &StackItem, n: u32) -> Result<(), Suspend> {
    g.discard_hand(item.controller);
    g.draw(item.controller, n);
    Ok(())
}

/// "Each player discards their hand, then draws seven cards": every player, in turn order from the active player
/// (CR 101.4), discards together and then draws; nobody is asked, a whole hand leaving nothing to choose. Wheel of Fortune
/// and Reforge the Soul print it.
pub fn each_player_discards_hand_then_draws_seven(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let hands: Vec<ObjectId> = players.iter().flat_map(|&p| g.state().zone(Zone::hand(p)).to_vec()).collect();
    g.discard_together(&hands);
    for p in players {
        g.draw(p, 7);
    }
    Ok(())
}

/// "You gain N life" (CR 119.3). Zuran Orb and Inventor's Fair print it (two and one).
pub fn gain_life(g: &mut dyn Game, item: &StackItem, n: i32) -> Result<(), Suspend> {
    g.gain_life(item.controller, n);
    Ok(())
}

/// "[This source] deals `n` damage to each opponent" (CR 120.3, 102.2): one damage event for each opponent still in the game,
/// dealt together (CR 120.2), by the resolving object as it last was if it has left (CR 113.7a). Chandra, Torch of Defiance,
/// Eye of Jace, Fiery Confluence, Impact Tremors, Warleader's Call, Vivi Ornitier, Gau, Keral Keep Disciples, Voldaren
/// Epicure and Quintorius Kand print it.
pub fn damage_each_opponent(g: &mut dyn Game, item: &StackItem, n: u16) {
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), n)).collect();
    g.damage_together(&hits);
}

/// "Surveil N" (CR 701.25) for the ability's controller. The surveil lands and Dragon's Rage Channeler print it.
pub fn surveil(g: &mut dyn Game, item: &StackItem, n: usize) -> Result<(), Suspend> {
    crate::surveil(g, item.controller, item.object, n)
}

/// "You become the monarch" (CR 725.1). Aragorn, King of Gondor and Court of Garenbrig print it.
pub fn become_monarch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.become_monarch(item.controller);
    Ok(())
}

/// "You take the initiative" (CR 726.2). Passageway Seer and Seasoned Dungeoneer print it.
pub fn take_the_initiative(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.take_initiative(item.controller);
    Ok(())
}

/// "Sacrifice this [permanent]" (CR 701.21): the ability's source. City of Traitors and Carnage, Crimson Chaos print it.
pub fn sacrifice_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.sacrifice(&[item.object]);
    Ok(())
}

/// "Destroy target [thing]" (CR 701.8): the ability's first target.
fn destroy_first_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    crate::destroy_target(g, item, 0);
    Ok(())
}

/// "{1}, Sacrifice this creature: Destroy target artifact or enchantment." Cathar Commando, Outland Liberator
/// (and its back face) and Shattered Acolyte print it.
pub const SACRIFICE_TO_DESTROY_ARTIFACT_OR_ENCHANTMENT: AbilityDef = ability(
    cost("{1}").and(SACRIFICE_THIS),
    destroy_first_target,
    "{1}, Sacrifice this creature: Destroy target artifact or enchantment.",
)
.targets(&[crate::targets::ARTIFACT_OR_ENCHANTMENT]);

/// "Transform this [permanent]" (CR 701.28): the ability's source. Aang, Swift Savior and Concealing Curtains print it.
pub fn transform_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.transform(item.object);
    Ok(())
}

/// "Untap this [permanent]" (CR 701.26): the ability's source. Basalt Monolith and Grim Monolith print it.
pub fn untap_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.untap(item.object);
    Ok(())
}

/// "You may [do something]" (CR 117.3, 609.3): asks `player` whether to, for `purpose`, and says whether they did. The
/// question is the usual `May`, so a scenario answers it with `Accept` or `Decline`. Ad Nauseam, Cori-Steel Cutter and Currency Converter ask
/// it.
pub fn ask_may(g: &mut dyn Game, player: PlayerId, source: ObjectId, purpose: Purpose) -> Result<bool, Suspend> {
    Ok(g.ask(player, RequestKind::May { source, purpose })? == Answer::Yes)
}

/// "You may return this card from your graveyard to ..." (CR 113.6k): `ask_may` for a card that triggered from a zone it
/// functions in, asked only while the card is still the object that triggered (CR 400.7). Once it has left, nothing is asked
/// and the answer is no. Squee, Goblin Nabob and Vengevine ask it.
pub fn ask_may_return_this_card(g: &mut dyn Game, item: &StackItem, purpose: Purpose) -> Result<bool, Suspend> {
    if !is_live(g, item.object) {
        return Ok(false);
    }
    ask_may(g, item.controller, item.object, purpose)
}

/// "Return target spell or permanent to its owner's hand": a spell leaves the stack for its owner's hand without being
/// countered (CR 400.3, so one that can't be countered goes too); a permanent is returned (CR 400.7: a token ceases to
/// exist, CR 111.7). Which it is is read from the stack. Jeskai Revelation and Sink into Stupor print it.
pub fn return_spell_or_permanent_to_hand(g: &mut dyn Game, object: ObjectId) {
    if g.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == object) {
        g.remove_spell(object, CounterTo::Hand);
    } else {
        g.return_to_hand(&[object]);
    }
}

/// "[This spell] deals `n` damage to each creature" (Pyroclasm, Fiery Confluence, Avengers Disassembled): one damage event
/// for every creature on the battlefield as the effect is carried out (CR 608.2c), dealt simultaneously (CR 120.2, 603.2c).
pub fn damage_each_creature(g: &mut dyn Game, item: &StackItem, n: u16) {
    let hits: Vec<(ObjectId, Target, u16)> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .map(|id| (item.object, Target::Object(id), n))
        .collect();
    g.damage_together(&hits);
}

/// "[This] deals `n` damage to that player": the player a trigger fired about (`Fired::about(Target::Player(..))`, its
/// subject), dealt by the ability's source, as it last was if it has left (CR 113.7a, 608.2h). Burning-Tree Shaman, Harsh
/// Mentor and Immolation Shaman print it.
pub fn damage_that_player(g: &mut dyn Game, item: &StackItem, n: u16) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.damage(item.object, Target::Player(p), n);
    }
    Ok(())
}

/// "`target` gains hexproof until end of turn" (CR 702.11b, 611.2a): a layer 6 keyword for the rest of the turn, from the
/// effect's source. Liliana the Faultless, Octopus Form, Shore Up and Snakeskin Veil print it.
pub fn gain_hexproof_until_end_of_turn(g: &mut dyn Game, source: ObjectId, target: ObjectId) {
    g.add_effect(source, target, EffectKind::AddKeywords(Keywords::of(&[Keyword::Hexproof])), Duration::EndOfTurn);
}

/// "It gains haste" with no duration (CR 611.2a, 613.1f): a layer 6 keyword for as long as the object stays the object it is
/// (a new one, CR 400.7, has none). Anzrag's Rampage, Sneak Attack, Through the Breach, Emperor of Bones, Goryo's Vengeance,
/// Earthbend and unearth print it.
pub fn gain_haste(g: &mut dyn Game, source: ObjectId, object: ObjectId) {
    g.add_effect(source, object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::Indefinite);
}

/// "`target` gains indestructible until end of turn" (CR 702.12b, 611.2a): a layer 6 keyword for the rest of the turn, from the
/// effect's source. Arwen, Mortal Queen, Cecil, Dark Knight, Crumb and Get It, Divine Resilience, Iron Shield Elf, Revitalizing Repast, Ursine
/// Monstrosity and Boros Charm print it.
pub fn gain_indestructible_until_end_of_turn(g: &mut dyn Game, source: ObjectId, target: ObjectId) {
    g.add_effect(source, target, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::EndOfTurn);
}

/// "`target` gains hexproof and indestructible until end of turn" (CR 702.11b, 702.12b, 611.2a): both keywords from one
/// effect, a layer 6 change for the rest of the turn. Invasion of Gobakhan's Lightshield Array, Spectacular Spider-Man
/// and Restoration Magic print it.
pub fn gain_hexproof_and_indestructible_until_end_of_turn(g: &mut dyn Game, source: ObjectId, target: ObjectId) {
    g.add_effect(source, target, EffectKind::AddKeywords(Keywords::of(&[Keyword::Hexproof, Keyword::Indestructible])), Duration::EndOfTurn);
}
