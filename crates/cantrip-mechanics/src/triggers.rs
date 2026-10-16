//! Trigger conditions and abilities many cards print.

use cantrip_api::*;

/// What ward's discard cost asks the player whose spell or ability targeted, the question on that spell or ability.
const WARD_DISCARD: Question =
    Question::new("DiscardUnlessCountered", "Discard a card, or {source} is countered?", "Ward: what it targeted can't be targeted for free", "Discard a card", "Let it be countered").no_default();

/// "When this enters" (CR 603.6a): the entering object's own zone change onto the battlefield.
pub fn this_enters(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Battlefield) => Fired::PLAIN,
        _ => None,
    }
}

/// "When this enters, if you cast it" (CR 603.4, 603.6a): the permanent entered as the spell that resolved, and that spell
/// was cast - the history says so. A permanent put onto the battlefield, or a copy of a permanent spell (a token, CR 707.10f),
/// was not cast. Lutri, The One Ring and Uldaros Theorix print it.
pub fn this_enters_if_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, to, reason: ZoneReason::Resolve, .. }
            if *new == source && to.is(ZoneKind::Battlefield) && q.history().cast_of(*old).is_some() =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// An intervening "if it's on the battlefield" (CR 603.4), checked as the ability triggers and again as it resolves: the
/// permanent is still the object that triggered (CR 400.7) and has not left. Animate Dead and Necromancy print it.
pub fn this_is_on_the_battlefield(q: &dyn Query, asker: &Asker) -> bool {
    q.is_on_battlefield(asker.source) && q.obj(asker.source).became.is_none()
}

/// "Whenever a token you control enters" / "whenever one or more tokens you control enter" (CR 603.6a): a token under the
/// source's controller's control as it entered, however it was made (created, or a copy of a permanent spell). The event's
/// subject is the token, so a batch that entered together is the subjects of the matching events (CR 603.2c). Belladonna
/// Took and Caretaker's Talent print it.
pub fn token_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.obj(*new).token && q.controller(*new) == me.controller =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// Landfall (CR 207.2c): a land entering the battlefield under your control, by any means. Icetill Explorer and
/// Tireless Tracker print it.
pub fn landfall(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == me.controller && q.chars(*new).is(CardType::Land) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "When you unlock this door" on the Room's right half (CR 709.5h): the Room permanent was given that half's unlocked
/// designation by its controller, whether as it entered because the half was cast (CR 709.5d) or after (CR 709.5e-f).
/// Ritual Chamber, Awakening Hall and Cadaver Lab print it. (The left half's is the same with `DOOR_LEFT`, added with the
/// first card that has one.)
pub fn you_unlock_this_right_door(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    you_unlock_this_door(q, source, event, DOOR_RIGHT)
}

fn you_unlock_this_door(q: &dyn Query, source: ObjectId, event: &Event, door: u8) -> Option<Fired> {
    match event {
        Event::DoorUnlocked { permanent, door: unlocked, player } if *permanent == source && *unlocked == door && *player == q.controller(source) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "When you cast this spell" (CR 603.2, 601.2i): the spell itself, from wherever it was cast. Sage of the Skies and
/// every cascade card print it.
pub fn cast_this_spell(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if *spell == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever you draw a card" (CR 120.3, 603.2): the source's controller drew. Sheoldred, the Apocalypse, Teferi, Hero of
/// Dominaria's emblem and Shang-Chi and the Ten Rings print it.
pub fn you_draw_a_card(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Drew { player, .. } if *player == me.controller => Fired::PLAIN,
        _ => None,
    }
}

/// "When the Nth [kind] counter is put on this permanent" (CR 122.6, 603.2): counters of kind `KIND` (`CounterKind::Plan
/// as u8`) were put on it and the Nth was among them - it had fewer than N before and has N or more now, as the event
/// records (`total`). So it triggers once however many are put at once (CR 603.2c), not for a counter after the Nth,
/// whatever a replacement made the number (doubling), for a permanent that enters with N or more of them (CR 122.6), and
/// again if counters were removed and the Nth is put once more, as a Saga's chapters count (CR 714.2b). Political
/// Triumph, Death to Our Enemies and the other Plans print it with plan counters; Shang-Chi and the Ten Rings with +1/+1
/// counters.
pub fn nth_counter_put_on_this<const KIND: u8, const N: i16>(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind, amount, total, .. } if *object == me.source && *kind as u8 == KIND && total - amount < N && *total >= N => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Cascade's effect (CR 702.85a): exile cards from the top of the library until a nonland card with a lesser mana
/// value than the spell's; its controller may cast it without paying its mana cost; then every card exiled and not
/// cast goes to the bottom of the library in a random order. The spell's mana value is its card's, X included, as it
/// is now or as it last was (CR 608.2h). Judged by the card rather than the face cast (an Adventure's cheaper face is
/// not considered).
fn cascade_resolves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let def = g.card(g.chars(item.object).card);
    let limit = def.cost.mana_value() + def.cost.x as u32 * g.obj(item.object).x as u32;

    let mut exiled: Vec<ObjectId> = Vec::new();
    let mut hit = None;
    while let Some(&top) = g.state().zone(Zone::library(player)).first() {
        let moved = g.exile(top);
        if !g.obj(moved).zone.is(ZoneKind::Exile) {
            continue;
        }
        exiled.push(moved);
        let chars = g.chars(moved);
        if !chars.is(CardType::Land) && g.card(chars.card).cost.mana_value() < limit {
            hit = Some(moved);
            break;
        }
    }
    // "You may cast it without paying its mana cost if the resulting spell's mana value is less than this spell's": the face
    // cast is the spell, so only a face under the limit may be (CR 702.85a, 709.3b, 715.3b).
    if let Some(card) = hit {
        let ways: Vec<CastWay> = g.cast_ways_during_resolution(card).into_iter().filter(|w| u32::from(w.mana_value) < limit).collect();
        if !ways.is_empty() && matches!(g.ask(player, RequestKind::May { source: card, purpose: Purpose::CastOrBottom })?, Answer::Yes)
            && let Some(way) = crate::choose::cast_way(g, player, card, ways)?
        {
            g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), way: Some(way.way), ..CastTerms::default() })?;
        }
    }

    // What was cast has left exile with its identity; the rest go to the bottom, shuffled among themselves.
    let rest: Vec<ObjectId> = exiled.into_iter().filter(|&c| is_live(g, c) && g.obj(c).zone.is(ZoneKind::Exile)).collect();
    g.put_into_library(&rest, LibraryPlace::BottomInRandomOrder);
    Ok(())
}

/// Annihilator N (CR 702.86a): "Whenever this creature attacks, defending player sacrifices N permanents." The defending
/// player is the one attacked, or the controller of the planeswalker or protector of the battle attacked (CR 508.5).
/// Emrakul, the Aeons Torn and Ulamog's Crusher print it.
pub const fn annihilator<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::ATTACKS, this_attacks_defender), annihilate::<N>, text)
}

/// "Whenever this creature attacks" (CR 508.1m, 603.2): its own attack, whatever it attacks. Aragorn, King of Gondor and
/// Caustic Bronco print it. The `listens` mask is `EventMask::ATTACKS`.
pub fn this_attacks(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever this creature attacks" where the ability is about whom it attacks: the player attacked, or the
/// planeswalker or battle (CR 508.5). The defender is the trigger's subject. Generous Plunderer and Otharri, Sun's
/// Glory print it.
pub fn this_attacks_defender(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender } if *attacker == source => Fired::about(*defender),
        _ => None,
    }
}

/// "Whenever equipped creature attacks" (CR 508.1m, 301.5): the creature this Equipment is attached to attacks. Adaptive
/// Omnitool and Argentum Armor print it. `equipped_creature_attacks_subject` carries the creature as the subject.
pub fn equipped_creature_attacks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.attached_to(source) == Some(*attacker) => Fired::PLAIN,
        _ => None,
    }
}

/// `equipped_creature_attacks` with the attacking creature as the trigger's subject, for an ability that does something
/// to it ("it gains ..."). Bone Sabres and Strength-Testing Hammer print it.
pub fn equipped_creature_attacks_subject(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if q.attached_to(source) == Some(*attacker) => Fired::about(Target::Object(*attacker)),
        _ => None,
    }
}

/// "Whenever enchanted player is attacked" (CR 508.1m, 303.4): a creature attacks the player this Curse is attached to,
/// directly. The attacking creature is the trigger's subject. Curse of Opulence and Curse of Predation print it.
pub fn enchanted_player_attacked(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, defender: Target::Player(p) } if q.obj(source).attached_player == Some(*p) => {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

/// "Whenever you attack" (CR 508.1m): the controller declares attackers, whatever their number. Guide of Souls and Inti,
/// Seneschal of the Sun print it.
pub fn you_attack(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::AttackersDeclared { player, .. } if *player == me.controller => Fired::PLAIN,
        _ => None,
    }
}

fn annihilate<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(defending) = g.defending_player(item.subject) else { return Ok(()) };
    crate::sacrifice_chosen(g, defending, item.object, N, |_| true)
}

/// Cascade (CR 702.85a): "When you cast this spell, exile cards from the top of your library until you exile a nonland
/// card that costs less. You may cast it without paying its mana cost. Put all cards exiled this way that weren't cast
/// on the bottom of your library in a random order." Bloodbraid Challenger and Bloodbraid Elf print it.
pub const CASCADE: AbilityDef = triggered(when(EventMask::SPELL_CAST, cast_this_spell), cascade_resolves, "Cascade")
    .anywhere();

fn cycle_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// Cycling [cost] (CR 702.29a): "[cost], Discard this card: Draw a card", from the hand. `text` is the ability's
/// full text, which a scenario's `Ability:` matches.
pub const fn cycling(mana: ManaCost, text: &'static str) -> AbilityDef {
    ability(Cost::mana(mana).and(DISCARD_THIS), cycle_draw, text)
        .from(ZoneKind::Hand)
}

fn lone_attacker(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. }
            if q.state().attackers.len() == 1 && q.controller(*attacker) == q.controller(source) =>
        {
            Fired::about(Target::Object(*attacker))
        }
        _ => None,
    }
}

fn exalted_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        crate::pump(g, item.object, creature, 1, 1);
    }
    Ok(())
}

/// Exalted (CR 702.83a): "Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn."
/// Ignoble Hierarch and Noble Hierarch print it word for word.
pub const EXALTED: AbilityDef = triggered(when(EventMask::ATTACKS, lone_attacker), exalted_pump, "Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.");

/// "When this creature enters, if its squad cost was paid": an unpaid cast, or a token copy, does not trigger at all.
fn entered_squadded(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Battlefield) && q.obj(source).kicked > 0 => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Create a token that's a copy of it for each time its squad cost was paid, all at once, from its copiable values
/// as the ability resolves.
fn squad_copies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let times = g.obj(item.object).kicked as u32;
    if times > 0 {
        g.create_token_copies(item.object, item.controller, times)?;
    }
    Ok(())
}

/// Squad (CR 702.157a): the enters trigger. Securitron Squadron and Sicarian Infiltrator print it word for word; the
/// additional cost is the card's `kicker`.
pub const SQUAD: AbilityDef = triggered(when(EventMask::ZONE_CHANGE, entered_squadded), squad_copies, "Squad: When this creature enters, if its squad cost was paid, create a token that's a copy of it for each time its squad cost was paid.");

fn ward_matches(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameTarget { target: Target::Object(t), by, controller } if *t == source && *controller != q.controller(source) => {
            // What targeted it, who controls that, and where it is on the stack (it is the top item as it is announced).
            Some(Fired {
                subject: Target::Object(*by),
                data: [controller.0 as u32, q.state().stack.len().saturating_sub(1) as u32],
            })
        }
        _ => None,
    }
}

fn ward_effect<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(by) = item.subject else { return Ok(()) };
    let payer = PlayerId(item.data[0] as u8);
    if !g.unless_pays(payer, item.object, by, generic(N))? {
        g.counter_item(by, item.data[1] as usize);
    }
    Ok(())
}

/// Ward {N} (CR 702.21a): "Whenever this becomes the target of a spell or ability an opponent controls, counter it
/// unless that player pays {N}." A creature granted ward (Lavaspur Boots) has it from the granting card's delayed
/// abilities; Kappa Cannoneer prints it with {4}.
pub const fn ward<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::BECAME_TARGET, ward_matches), ward_effect::<N>, text)
        .ward()
}

/// What ward's life cost asks the player whose spell or ability targeted, the question on that spell or ability: pay it,
/// or have it countered.
fn ward_life_question(n: u8) -> &'static Question {
    const DETAIL: &str = "Ward: what it targeted can't be targeted for free";
    const TWO: Question =
        Question::new("PayLifeUnlessCountered", "Pay 2 life, or {source} is countered?", DETAIL, "Pay 2 life", "Let it be countered").amount(2).no_default();
    const THREE: Question =
        Question::new("PayLifeUnlessCountered", "Pay 3 life, or {source} is countered?", DETAIL, "Pay 3 life", "Let it be countered").amount(3).no_default();
    const ANY: Question =
        Question::new("PayLifeUnlessCountered", "Pay the life, or {source} is countered?", DETAIL, "Pay the life", "Let it be countered").no_default();
    match n {
        2 => &TWO,
        3 => &THREE,
        _ => &ANY,
    }
}

fn ward_life_effect<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(by) = item.subject else { return Ok(()) };
    let payer = PlayerId(item.data[0] as u8);
    // A cost of life is paid only by a player with that much (CR 119.4), so nobody else is asked; the payer says whether
    // to pay, as the C# engine's unlessPaysLife asked it.
    let paid = g.state().player(payer).life >= N as i32
        && g.ask(payer, RequestKind::May { source: by, purpose: Purpose::Ask(ward_life_question(N)) })? == Answer::Yes;
    if paid {
        g.lose_life(payer, N as i32);
    } else {
        g.counter_item(by, item.data[1] as usize);
    }
    Ok(())
}

/// Ward--Pay N life (CR 702.21a): "Whenever this becomes the target of a spell or ability an opponent controls, counter it
/// unless that player pays N life." Sedgemoor Witch prints it with 3.
pub const fn ward_life<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::BECAME_TARGET, ward_matches), ward_life_effect::<N>, text)
        .ward()
}

fn ward_discard_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(by) = item.subject else { return Ok(()) };
    let payer = PlayerId(item.data[0] as u8);
    // A player with no card in hand cannot pay and is not asked; one who can is asked whether to, and then which card.
    if !g.state().zone(Zone::hand(payer)).is_empty()
        && matches!(g.ask(payer, RequestKind::May { source: by, purpose: Purpose::Ask(&WARD_DISCARD) })?, Answer::Yes)
    {
        crate::discard_chosen(g, payer, item.object, 1)?;
        return Ok(());
    }
    g.counter_item(by, item.data[1] as usize);
    Ok(())
}

/// Ward--Discard a card (CR 702.21a, 118.12): "Whenever this becomes the target of a spell or ability an opponent controls,
/// counter it unless that player discards a card." Sunset Saboteur prints it.
pub const fn ward_discard(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::BECAME_TARGET, ward_matches), ward_discard_effect, text)
        .ward()
}

/// "Whenever you cast a noncreature spell" (CR 603.2): prowess, Displacer Kitten and Dragon's Rage Channeler print it.
pub fn noncreature_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && !q.chars(*spell).is(CardType::Creature) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn prowess_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    crate::pump(g, item.object, item.object, 1, 1);
    Ok(())
}

/// Prowess (CR 702.108a): "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn", as a
/// real triggered ability. The Otter token of Stormchaser's Talent and the Monk token of Cori-Steel Cutter carry it.
pub const PROWESS: AbilityDef = triggered(when_asked(EventMask::SPELL_CAST, noncreature_spell_you_cast), prowess_pump, "Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)");

/// The instant or sorcery spell `player` cast or copied, if this event is one (CR 601.2i, 707.10): magecraft's trigger event,
/// and what Veyran, Voice of Duality's "you casting or copying an instant or sorcery spell" names. A copy of a spell is not
/// cast (CR 707.10a), and a copy of a card that is no spell on the stack is not "copying a spell", so only a copy on the stack
/// counts. The spell is read as it is on the stack.
pub fn instant_or_sorcery_cast_or_copied(q: &dyn Query, event: &Event) -> Option<(ObjectId, PlayerId)> {
    let (spell, player) = match *event {
        Event::SpellCast { spell, player, .. } => (spell, player),
        Event::Copied { copy, player, .. } if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == copy) => (copy, player),
        _ => return None,
    };
    let c = q.chars(spell);
    (c.is(CardType::Instant) || c.is(CardType::Sorcery)).then_some((spell, player))
}

/// Magecraft (CR 207.2c, an ability word): "Whenever you cast or copy an instant or sorcery spell", one ability that triggers
/// on either. Witherbloom Apprentice, Sedgemoor Witch and Veyran, Voice of Duality print it.
pub fn you_cast_or_copy_instant_or_sorcery(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match instant_or_sorcery_cast_or_copied(q, event) {
        Some((_, player)) if player == me.controller => Fired::PLAIN,
        _ => None,
    }
}

/// "When this is put into a graveyard from the battlefield" (CR 700.4 for a creature: "dies"), looked back on: the
/// ability is the object's as it last existed there (CR 603.10a). Use with `look_back: true`.
pub fn this_dies(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. } if *old == source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "When this leaves the battlefield" (CR 603.6c, 603.10a), looked back on whichever zone it goes to. Use with
/// `look_back: true`. Animate Dead and Cryogen Relic print it.
pub fn this_leaves(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. } if *old == source && from.is(ZoneKind::Battlefield) => Fired::PLAIN,
        _ => None,
    }
}

/// "At the beginning of your [step]" (CR 503.1a, 603.2): `step` began on its controller's turn.
fn your_step(q: &dyn Query, me: &Asker, event: &Event, step: Step) -> Option<Fired> {
    match event {
        Event::StepBegan { step: began } if *began == step && q.state().turn.active == me.controller => Fired::PLAIN,
        _ => None,
    }
}

/// "At the beginning of your upkeep" (CR 503.1a). Dark Confidant and Court of Garenbrig print it.
pub fn your_upkeep(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    your_step(q, me, event, Step::Upkeep)
}

/// "At the beginning of combat on your turn" (CR 507.1). Luminarch Aspirant and Ouroboroid print it.
pub fn beginning_of_your_combat(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    your_step(q, me, event, Step::BeginCombat)
}

/// "At the beginning of your end step" (CR 513.1). Dawn of a New Age and Feywild Caretaker print it.
pub fn your_end_step(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    your_step(q, me, event, Step::End)
}

/// "At the beginning of each end step" (CR 513.1, 603.2): any player's end step began. Gau, Feral Youth and Octomancer print it.
pub fn each_end_step(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::End } => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever a creature you control enters" (CR 603.6a): any creature, this one included, under your control as it
/// entered. Aura Shards and Warleader's Call print it.
pub fn creature_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == me.controller && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever another creature you control enters" (CR 603.6a): a creature other than this permanent, under your control
/// as it entered. The entering creature is the trigger's subject. Aunt May and Guide of Souls print it.
pub fn another_creature_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != me.source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Whenever a creature dies" (CR 700.4, 603.10a), looked back on: any creature, anyone's, as it last was on the battlefield.
/// The ability carries the player who then controlled it (CR 608.2h). Use with `look_back: true`. Fecundity and Ashes of the
/// Abhorrent print it.
pub fn creature_dies(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.chars(*old).is(CardType::Creature) =>
        {
            Fired::about(Target::Player(q.controller(*old)))
        }
        _ => None,
    }
}

/// "Whenever a creature you control dies" (CR 700.4, 603.10a), looked back on: the creature as it last was on the
/// battlefield, under the controller it last had. Use with `look_back: true`. Grave Venerations and Warhost's Frenzy
/// print it.
pub fn creature_you_control_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.chars(*old).is(CardType::Creature)
                && q.obj(*old).controller == me.controller =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever you cast a creature spell" (CR 603.2): its controller's spells only, judged on the spell as it is on the stack
/// (CR 601.2i), so a card that is a creature only there counts. Bontu's Monument, Glimpse of Nature, Silk, Web Weaver and
/// Spider-Man India print it.
pub fn creature_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && q.chars(*spell).is(CardType::Creature) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever you cast an artifact spell" (CR 603.2). Pinnacle Emissary and Ravenous Robots print it.
pub fn artifact_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && q.chars(*spell).is(CardType::Artifact) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// `deals_combat_damage_to_player` for an ability about the damaged player or the damage: the player is the trigger's
/// subject and the amount of damage its `data[0]`. Etali, Primal Conqueror and Fallen Shinobi print it.
pub fn deals_combat_damage_to_player_subject(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(p), amount, combat: true } if *from == source && *amount > 0 => {
            Some(Fired { subject: Target::Player(*p), data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "Whenever this creature deals combat damage to a player" (CR 510.2, 603.2): its own damage, greater than zero.
/// Bejeweled Warg prints it; Carnage, Crimson Chaos's granted ability has the same condition.
pub fn deals_combat_damage_to_player(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(_), amount, combat: true } if *from == source && *amount > 0 => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Whenever a creature you control deals combat damage to a player" (CR 510.2, 603.2): one trigger for each creature and
/// player, with "that player" fixed as the trigger's subject. Vraska, Golgari Queen's emblem and Enduring Curiosity print it.
pub fn creature_you_control_deals_combat_damage_to_player(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(p), amount, combat: true }
            if *amount > 0 && q.controller(*from) == me.controller && q.chars(*from).is(CardType::Creature) =>
        {
            Fired::about(Target::Player(*p))
        }
        _ => None,
    }
}

/// "Whenever you flip a coin" (CR 705.1): any flip its controller makes, won or lost.
pub fn you_flip_a_coin(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::CoinFlipped { player, .. } if *player == me.controller => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever you win a coin flip" (CR 705.2): a flip its controller wins.
pub fn you_win_a_flip(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::CoinFlipped { player, won: true } if *player == me.controller => Fired::PLAIN,
        _ => None,
    }
}

/// "Whenever you roll a die" (CR 706.2): a die its controller rolls, whatever its sides.
pub fn you_roll_a_die(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::DieRolled { player, .. } if *player == me.controller => Fired::PLAIN,
        _ => None,
    }
}

/// An activation (CR 602.2) as the event a "whenever ... activate(s) an ability" trigger sees: the object whose ability
/// it is, the player who activated it, and the ability. The object is read as it is, or as it last was if paying the cost
/// took it away (CR 113.7a, 608.2h). Only abilities that use the stack are activations triggers see (`EventMask::of`).
/// Ajani Unrelenting, Burning-Tree Shaman, Harsh Mentor, Immolation Shaman and Keral Keep Disciples ask it.
pub fn activation(q: &dyn Query, event: &Event) -> Option<(ObjectId, PlayerId, &'static ActivatedDef)> {
    let Event::AbilityActivated { source, ability, player } = *event else { return None };
    q.activated_ability(ability).map(|def| (source, player, def))
}

/// "Whenever you activate a loyalty ability [of ...]" (CR 606.2, 603.2): the permanent whose loyalty ability the ability's
/// controller (`me`, CR 603.7d for a delayed one) activated, for a condition on it ("of a Chandra planeswalker"), read as it was when the costs were paid or
/// as it last was if paying them took it away. Ajani Unrelenting and Keral Keep Disciples print it.
pub fn loyalty_ability_you_activated(q: &dyn Query, me: &Asker, event: &Event) -> Option<ObjectId> {
    let (object, player, def) = activation(q, event)?;
    (def.is_loyalty_ability() && player == me.controller).then_some(object)
}

/// "Whenever you activate a loyalty ability": any permanent's, by this ability's controller (Ajani Unrelenting).
pub fn you_activate_a_loyalty_ability(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    loyalty_ability_you_activated(q, me, event).and(Fired::PLAIN)
}

/// "Whenever an opponent activates an ability of an artifact, creature, or land [on the battlefield] that isn't a mana
/// ability, ... to that player" (CR 602.2, 603.2): a permanent's ability (one that functions on the battlefield, not
/// cycling from a hand or embalm from a graveyard), the permanent an artifact, creature or land as it was when the costs
/// were paid, a sacrificed one as it last was on the battlefield. The ability is about the activating player. Harsh
/// Mentor and Immolation Shaman print it.
pub fn opponent_activates_ability_of_artifact_creature_or_land(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let (object, player, def) = activation(q, event)?;
    let types = q.chars(object);
    let permanent = def.from == ZoneKind::Battlefield
        && (types.is(CardType::Artifact) || types.is(CardType::Creature) || types.is(CardType::Land));
    let opponent = q.state().opponents(me.controller).any(|p| p == player);
    if permanent && opponent && !def.is_mana_ability() { Fired::about(Target::Player(player)) } else { None }
}

// --- The triggers, each with the events it listens to: `triggered(THIS_ENTERS, effect, text)` ---------------------

pub const THIS_ENTERS: Trigger = when(EventMask::ZONE_CHANGE, this_enters);
pub const THIS_ENTERS_IF_CAST: Trigger = when(EventMask::ZONE_CHANGE, this_enters_if_cast);
pub const LANDFALL: Trigger = when_asked(EventMask::ZONE_CHANGE, landfall);
pub const YOU_UNLOCK_THIS_RIGHT_DOOR: Trigger = when(EventMask::DOOR_UNLOCKED, you_unlock_this_right_door);
pub const YOU_DRAW_A_CARD: Trigger = when_asked(EventMask::DREW, you_draw_a_card);
pub const CAST_THIS_SPELL: Trigger = when(EventMask::SPELL_CAST, cast_this_spell);
pub const MAGECRAFT: Trigger = when_asked(EventMask(EventMask::SPELL_CAST.0 | EventMask::COPIED.0), you_cast_or_copy_instant_or_sorcery);
pub const THIS_ATTACKS: Trigger = when(EventMask::ATTACKS, this_attacks);
pub const THIS_ATTACKS_DEFENDER: Trigger = when(EventMask::ATTACKS, this_attacks_defender);
pub const EQUIPPED_CREATURE_ATTACKS: Trigger = when(EventMask::ATTACKS, equipped_creature_attacks);
pub const EQUIPPED_CREATURE_ATTACKS_SUBJECT: Trigger = when(EventMask::ATTACKS, equipped_creature_attacks_subject);
pub const ENCHANTED_PLAYER_ATTACKED: Trigger = when(EventMask::ATTACKS, enchanted_player_attacked);
pub const YOU_ATTACK: Trigger = when_asked(EventMask::ATTACKERS_DECLARED, you_attack);
pub const NONCREATURE_SPELL_YOU_CAST: Trigger = when_asked(EventMask::SPELL_CAST, noncreature_spell_you_cast);
pub const THIS_DIES: Trigger = when(EventMask::ZONE_CHANGE, this_dies);
pub const THIS_LEAVES: Trigger = when(EventMask::ZONE_CHANGE, this_leaves);
pub const YOUR_UPKEEP: Trigger = when_asked(EventMask::STEP_BEGAN, your_upkeep);
pub const BEGINNING_OF_YOUR_COMBAT: Trigger = when_asked(EventMask::STEP_BEGAN, beginning_of_your_combat);
pub const YOUR_END_STEP: Trigger = when_asked(EventMask::STEP_BEGAN, your_end_step);
pub const EACH_END_STEP: Trigger = when(EventMask::STEP_BEGAN, each_end_step);
pub const CREATURE_YOU_CONTROL_ENTERS: Trigger = when_asked(EventMask::ZONE_CHANGE, creature_you_control_enters);
pub const CREATURE_DIES: Trigger = when(EventMask::ZONE_CHANGE, creature_dies);
pub const CREATURE_YOU_CONTROL_DIES: Trigger = when_asked(EventMask::ZONE_CHANGE, creature_you_control_dies);
pub const ARTIFACT_SPELL_YOU_CAST: Trigger = when_asked(EventMask::SPELL_CAST, artifact_spell_you_cast);
pub const CREATURE_SPELL_YOU_CAST: Trigger = when_asked(EventMask::SPELL_CAST, creature_spell_you_cast);
pub const DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT: Trigger = when(EventMask::DAMAGE, deals_combat_damage_to_player_subject);
pub const DEALS_COMBAT_DAMAGE_TO_PLAYER: Trigger = when(EventMask::DAMAGE, deals_combat_damage_to_player);
pub const OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND: Trigger =
    when_asked(EventMask::ABILITY_ACTIVATED, opponent_activates_ability_of_artifact_creature_or_land);
pub const YOU_ACTIVATE_A_LOYALTY_ABILITY: Trigger = when_asked(EventMask::ABILITY_ACTIVATED, you_activate_a_loyalty_ability);
