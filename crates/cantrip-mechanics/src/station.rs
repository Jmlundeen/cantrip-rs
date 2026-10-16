//! Station (CR 702.184a): "Tap another untapped creature you control: Put a number of charge counters on this permanent
//! equal to the tapped creature's power. Activate only as a sorcery." The station symbols of a station card (CR 721.2) are
//! its own statics, read from its charge counters with `charged::<N>`: "{N+}[abilities]" means "as long as this permanent
//! has N or more charge counters on it, it has [abilities]" (CR 721.2a), and "{N+}[abilities][P/T]" makes it also "a
//! creature with base power and toughness [P/T] in addition to its other types" (CR 721.2b, `creature::<P, T>`).
//!
//! Every station card prints it: the Planets (Adagia, Windswept Bastion; Uthros, Titanic Godcore; Evendo, Waking Haven)
//! and the Spacecraft (Wurmwall Sweeper, Galvanizing Sawship, ...).

use cantrip_api::*;

/// "Tap another untapped creature you control" (CR 702.184a): not the station permanent itself, even once it is a creature.
fn another_untapped_creature_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.controller(id) == asker.controller && !q.obj(id).tapped && q.chars(id).is(CardType::Creature)
}

/// "Put a number of charge counters on this permanent equal to the tapped creature's power": its power as the ability
/// resolves, or as it last existed on the battlefield if it has left (CR 608.2h; the station rulings). A power of 0 or less
/// puts none and removes none (ruling 2025-07-25). A station permanent that has left the battlefield gets none
/// (`add_counters` skips an object that has changed zones: one that came back is a new object, CR 400.7).
fn put_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&tapped) = g.cost_objects(item).first() else { return Ok(()) };
    let power = g.chars(tapped).power.unwrap_or(0);
    if power > 0 {
        g.add_counters(item.object, CounterKind::Charge, power);
    }
    Ok(())
}

/// The station ability (CR 702.184a). Its cost taps a creature, not the {T} symbol (CR 107.5, 302.6), so a creature that
/// came under its controller's control this turn may station. `text` is the ability's full text, as printed.
pub const fn ability(text: &'static str) -> AbilityDef {
    cantrip_api::ability(Cost::tap_untapped(1, another_untapped_creature_you_control), put_charge_counters, text).sorcery_speed()
}

/// The number of charge counters on `source`, as the layers see it.
pub fn charge(view: &dyn LayerQuery, source: ObjectId) -> i16 {
    view.state().obj(source).counters(CounterKind::Charge)
}

/// A station symbol "{N+}" (CR 721.2a): the static ability of its striation applies to its own permanent while that
/// permanent has N or more charge counters on it. Use it as a static ability's `affects`.
pub fn charged<const N: i16>(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && charge(view, source) >= N
}

/// "[P/T]" in a striation (CR 721.2b), layer 4: it is a creature in addition to its other types.
pub fn is_a_creature(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Creature);
}

/// "[P/T]" in a striation (CR 721.2b), layer 7b: its base power and toughness. A station card has no power or toughness
/// otherwise (CR 721.2c), so its content file prints none.
pub fn base_pt<const P: i16, const T: i16>(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(P);
    chars.toughness = Some(T);
}

/// The abilities of a striation (CR 721.2a), layer 6: the ability at `INDEX` of the station card's `delayed` list. A
/// permanent that copies the station card has it too, read from its own charge counters (ruling 2025-07-25).
pub fn has<const INDEX: u8>(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(chars.card, INDEX));
    }
}
