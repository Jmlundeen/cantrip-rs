//! One-line effects as functions a `TriggeredDef` or `ActivatedDef` can name, parameterised the way `ward::<N>` is, so
//! a card does not write a wrapper around a single `Game` call. Each acts for the ability's controller or its source.

use cantrip_api::*;

/// "This creature enters with X +1/+1 counters on it" (CR 614.1c), for the X it was cast with; put onto the battlefield
/// without being cast, X is 0. Walking Ballista, Guiding Hydra and the ravenous creatures print it.
pub fn enters_with_x_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += entering.x as i16;
    Ok(())
}

/// "This creature enters with `N` -1/-1 counters on it" (CR 614.1c). Glen Elendra Guardian, Flitterwing Nuisance (one)
/// and Moonshadow (six) print it.
pub fn enters_with_minus_one_counters<const N: i16>(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::MinusOne as usize] += N;
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

/// "Creatures you control gain trample and get +P/+T until end of turn" (CR 611.2c): `pump_creatures_you_control` and trample for
/// each of them, the creatures being those on the battlefield as it resolves. The creatures, for what the rest of the sentence does
/// to them. Craterclaw Colossus (+X/+0, X the artifacts you control) and Craterhoof Behemoth (+X/+X, X the creatures) print it.
pub fn pump_and_trample_creatures_you_control(g: &mut dyn Game, item: &StackItem, power: i16, toughness: i16) -> Vec<ObjectId> {
    let creatures = pump_creatures_you_control(g, item, power, toughness);
    for &creature in &creatures {
        crate::trample_until_end_of_turn(g, item.object, creature);
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

/// "Target player draws N cards" (CR 121.1): the player the item targeted in `slot`, if the target is still legal (CR 608.2b).
/// Ancestral Recall (three), Flame of Anor and Ashling's Command (two) print it.
pub fn target_player_draws(g: &mut dyn Game, item: &StackItem, slot: usize, n: u32) {
    if let Some(player) = crate::targets::player(g, item, slot) {
        g.draw(player, n);
    }
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
    if may_discard_a_card(g, item)? {
        g.draw(item.controller, n);
    }
    Ok(())
}

/// "You may discard a card" (CR 701.9): the controller chooses a card of their hand to discard or none, as the ability
/// resolves and not as a cost, so nothing is asked of an empty hand. Whether a card was chosen and discarded, for the "if you
/// do" or the reflexive "when you do" (CR 603.12) that follows. `may_discard_to_draw`, Inti, Seneschal of the Sun and
/// Awaken the Honored Dead use it.
pub fn may_discard_a_card(g: &mut dyn Game, item: &StackItem) -> Result<bool, Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    let Some(&card) = crate::choose::cards(g, item.controller, item.object, Purpose::Discard, hand, 1, true)?.first() else { return Ok(false) };
    g.discard(card);
    Ok(true)
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
pub fn destroy_first_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
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

/// "Transform this [permanent]" (CR 701.27a): the ability's source, which does nothing if it has transformed since the
/// ability was put on the stack or, for a delayed ability, created (CR 701.27f). Aang, Swift Savior, Concealing Curtains,
/// Norman Osborn, Temple of the Dead and Aang, at the Crossroads's delayed ability print it.
pub fn transform_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.transform_by(item.object, item);
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

const RETURN_TO_BATTLEFIELD: Question = Question::new("ReturnToBattlefield", "return it from your graveyard to the battlefield?", "", "Return it", "Leave it");

/// "You may return this card from your graveyard to the battlefield" (CR 113.6k): `ask_may_return_this_card` with the
/// question worded for the battlefield, and the card put onto the battlefield untapped under its controller on a yes.
/// Nothing is asked once it is no longer the object that triggered (CR 400.7). Vengevine and Bloodghast print it.
pub fn may_return_this_card_to_battlefield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may_return_this_card(g, item, Purpose::Ask(&RETURN_TO_BATTLEFIELD))? {
        g.put_onto_battlefield(item.object, item.controller, false)?;
    }
    Ok(())
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

/// "Return target [permanent] to its owner's hand" (CR 400.3): the ability's or
/// spell's first target, nothing at all if it is no longer a legal target (CR 608.2b). A token that is returned ceases to
/// exist (CR 111.7). Aether Spellbomb, Banishing Knack, Karakas, Mystic Confluence, Brazen Borrower's Petty Theft and
/// Bounce Off print it, and, for a card in a graveyard, Bala Ged Recovery, Emeritus of Abundance and Auriok Salvagers.
pub fn return_first_target_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = crate::targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    Ok(())
}

/// "Change the target of target spell or ability with a single target" (CR 115.7a): the first target of the resolving spell or
/// ability is a spell or ability on the stack whose target its controller changes to another legal one as this resolves, and
/// the target is not changed if there is no other legal choice. An item whose only target is no longer legal is never
/// resolved (CR 608.2b), so nothing is checked here. Untimely Malfunction and Bolt Bend print it.
pub fn change_first_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.change_target(item.targets[0], item.controller)
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

/// "`target` gains lifelink until end of turn" (CR 702.15, 611.2a): a layer 6 keyword for the rest of the turn, from the
/// effect's source. Gix's Command and Archenemy's Charm print it.
pub fn gain_lifelink_until_end_of_turn(g: &mut dyn Game, source: ObjectId, target: ObjectId) {
    g.add_effect(source, target, EffectKind::AddKeywords(Keywords::of(&[Keyword::Lifelink])), Duration::EndOfTurn);
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

/// "A creature or planeswalker with the greatest mana value among creatures and planeswalkers they control": the
/// permanents `player` controls now that tie for the greatest mana value (`Query::mana_value`: tokens, copies and face-down
/// permanents read as the rules say) in the one group of their creatures and planeswalkers, so a planeswalker of 5 outranks a
/// creature of 3. The player chooses among them (CR 701.21a, 608.2d). Break Under Pressure sacrifices one of them; End of
/// the Hunt exiles one.
pub fn greatest_mana_value_creature_or_planeswalker(g: &dyn Game, player: PlayerId) -> Vec<ObjectId> {
    let group: Vec<ObjectId> = g
        .permanents_of(player)
        .into_iter()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) || chars.is(CardType::Planeswalker)
        })
        .collect();
    let greatest = group.iter().map(|&id| g.mana_value(id)).max().unwrap_or(0);
    group.into_iter().filter(|&id| g.mana_value(id) == greatest).collect()
}
