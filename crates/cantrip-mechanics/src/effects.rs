//! One-line effects as functions a `TriggeredDef` or `ActivatedDef` can name, parameterised the way `ward::<N>` is, so
//! a card does not write a wrapper around a single `Game` call. Each acts for the ability's controller or its source.

use cantrip_api::*;

/// "Draw N cards" (CR 121.1). Chromatic Star and Mind Stone print "draw a card"; Coveted Jewel and Bankrupt in Blood draw
/// three.
/// "Put `n` `kind` counters on each creature you control": on the creatures `player` controls as it is read, one after the
/// other. The creatures, for what the rest of the sentence does to "those creatures". Abandoned Air Temple, Elspeth,
/// Storm Slayer, Anduril, Narsil Reforged and The Theorist, Jace Beleren print it.
pub fn counters_on_creatures_you_control(g: &mut dyn Game, player: PlayerId, kind: CounterKind, n: i16) -> Vec<ObjectId> {
    let creatures: Vec<ObjectId> = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for &id in &creatures {
        g.add_counters(id, kind, n);
    }
    creatures
}

pub fn draw<const N: u32>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, N);
    Ok(())
}

/// "Draw a card, then discard a card" (CR 121.1, 701.9): the controller draws, then chooses a card of their hand and
/// discards it. Currency Converter and Sphinx of False Conclusions print it.
pub fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    crate::discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

/// "You gain N life" (CR 119.3). Zuran Orb and Inventor's Fair print it (two and one).
pub fn gain_life<const N: i32>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, N);
    Ok(())
}

/// "Surveil N" (CR 701.25) for the ability's controller. The surveil lands and Dragon's Rage Channeler print it.
pub fn surveil<const N: usize>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    crate::surveil(g, item.controller, item.object, N)
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
